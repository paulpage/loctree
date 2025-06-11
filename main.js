const E = document.createElement.bind(document)

const filters = {
    languages: {},
}

const collapseCache = new Map()

let isAllExpanded = false

function collapseAll() {
    isAllExpanded = false
    collapseCache.clear()
    buildTree()
}

function expandAll() {
    isAllExpanded = true
    buildTree()
}

function addToNode(node, e, path) {
    if (filters.languages[e.language]) {

        e.language in node.stats || (node.stats[e.language] = {
            code: 0,
            comments: 0,
            blanks: 0,
        })
        node.stats[e.language].code += e.code
        node.stats[e.language].comments += e.comments
        node.stats[e.language].blanks += e.blanks

        if (path.length > 0) {
            path[0] in node.children || (node.children[path[0]] = {
                stats: {},
                children: {},
            })
            addToNode(node.children[path[0]], e, path.slice(1))
        }
    }
}

function htmlWriteNode(html, node, level = 0, key = "node::") {
    for ([name, child] of Object.entries(node.children)) {
        let code = 0, comments = 0, blanks = 0;
        for ([lang, stats] of Object.entries(child.stats)) {
            if (filters.languages[lang]) {
                code += stats.code
                comments += stats.comments
                blanks += stats.blanks
            }
        }

        if (code + comments + blanks === 0) continue;

        const msg = E('span')
        const bold = E('b')
        bold.textContent = name
        msg.appendChild(bold)
        msg.appendChild(document.createTextNode(`: ${code} code, ${comments} comments, ${blanks} blanks`))

        if (child.children && Object.keys(child.children).length > 0) {
            const details = E('details')
            childKey = key + "/" + name
            details.id = childKey
            if (level === 0 || collapseCache.get(childKey) || isAllExpanded) {
                details.open = true;
            }
            details.addEventListener("toggle", () => {
                collapseCache.set(details.id, details.open)
            })

            const summary = E('summary')
            summary.appendChild(msg)
            details.appendChild(summary)

            const childElement = htmlWriteNode(E("span"), child, level + 1, childKey)

            details.appendChild(childElement)
            html.appendChild(details)
        } else {
            const p = E('p')
            p.appendChild(msg)
            html.appendChild(p);
        }
    }
    return html
}

function buildFilters() {
    for (e of data) {
        filters.languages[e.language] = true
    }

    const container = document.getElementById("filters")

    {
        const wrapper = E("div")
        wrapper.className = 'checkbox-wrapper'

        const checkbox = E("input")
        checkbox.type = 'checkbox'
        checkbox.id = 'chk-all'
        checkbox.checked = true
        checkbox.addEventListener('change', () => {
            Object.keys(filters.languages).forEach(key => {
                filters.languages[key] = checkbox.checked
                document.getElementById("chk-" + key).checked = checkbox.checked
            })
            buildTree()
        })

        const label = E('label')
        label.htmlFor = checkbox.id
        label.textContent = "(All)"

        wrapper.appendChild(checkbox)
        wrapper.appendChild(label)

        container.appendChild(wrapper)
    }

    Object.keys(filters.languages).forEach(text => {
        const wrapper = E("div")
        wrapper.className = 'checkbox-wrapper'

        const checkbox = E("input")
        checkbox.type = "checkbox"
        checkbox.id = `chk-${text}`
        checkbox.checked = filters.languages[text]
        checkbox.addEventListener('change', () => {
            filters.languages[text] = checkbox.checked;
            console.log(filters) // TODO remove
            buildTree()
        })

        const label = E('label')
        label.htmlFor = checkbox.id
        label.textContent = text

        wrapper.appendChild(checkbox)
        wrapper.appendChild(label)

        container.appendChild(wrapper)
    })

}

function buildTree() {
    const start = performance.now()

    var tree = {
        stats: {},
        children: {},
    }

    for (e of data) {
        path = e.path.split("\\")
        addToNode(tree, e, path)
    }

    const treeBuilt = performance.now()

    html = htmlWriteNode(E("span"), tree)
    tree = document.getElementById("tree")
    tree.innerHTML = ""
    tree.appendChild(html)

    const end = performance.now()
    // const treeBuildTime = 
    // const elapsed = performance.now() - start
    console.log("tree build: " + (treeBuilt - start) + "ms")
    console.log("dom build: " + (end - treeBuilt) + "ms")
    console.log("total: " + (end - start) + "ms")
}

document.addEventListener("DOMContentLoaded", function() {
    buildFilters()
    buildTree()
})
