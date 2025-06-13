const E = document.createElement.bind(document)

const filters = {
    languages: {},
}

const I_LANG = 0
const I_PATH = 1
const I_CODE = 2
const I_COMMENTS = 3
const I_BLANKS = 4


const collapseCache = new Map()

let g_tree = {}

let isAllExpanded = false

function collapseAll() {
    isAllExpanded = false
    collapseCache.clear()
    render()
}

function expandAll() {
    isAllExpanded = true
    render()
}

function addToNode(node, e, path) {
    if (filters.languages[e[I_LANG]]) {
        if ("stats" in node) {
            e[I_LANG] in node.stats || (node.stats[e[I_LANG]] = [0, 0, 0])
            node.stats[e[I_LANG]][0] += e[I_CODE]
            node.stats[e[I_LANG]][1] += e[I_COMMENTS]
            node.stats[e[I_LANG]][2] += e[I_BLANKS]

            if (path.length > 0) {
                path[0] in node.children || (node.children[path[0]] = {
                    stats: {},
                    children: {},
                })
                addToNode(node.children[path[0]], e, path.slice(1))
            }
        }
    }
}

function getTotalCode(node) {
    let total = 0
    for (const [lang, stats] of Object.entries(node.stats)) {
        if (filters.languages[lang]) {
            total += stats[0]
        }
    }
    return total
}

function sortTreeRecursive(node) {
    node.children = Object.fromEntries(
        Object.entries(node.children)
            .sort((a, b) => getTotalCode(b[1]) - getTotalCode(a[1]))
    )
    Object.values(node.children).forEach(sortTreeRecursive)
}

function htmlWriteNode(html, node, level = 0, key = "node::") {
    for ([name, child] of Object.entries(node.children)) {
        let code = 0, comments = 0, blanks = 0;
        for ([lang, stats] of Object.entries(child.stats)) {
            if (filters.languages[lang]) {
                code += stats[0]
                comments += stats[1]
                blanks += stats[2]
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
            const summary = E('summary')
            summary.appendChild(msg)
            details.appendChild(summary)
            if (level === 0 || collapseCache.get(childKey) || isAllExpanded) {
                details.open = true;
            }
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
        filters.languages[e[I_LANG]] = true
    }

    const container = document.getElementById("filters")

    {
        const wrapper = E("div")
        wrapper.className = 'checkbox-wrapper'

        const checkbox = E("input")
        checkbox.type = 'checkbox'
        checkbox.id = 'chk-all'
        checkbox.checked = true
        checkbox.setAttribute('data-form-type', 'other')
        checkbox.addEventListener('change', () => {
            Object.keys(filters.languages).forEach(key => {
                filters.languages[key] = checkbox.checked
                document.getElementById("chk-" + key).checked = checkbox.checked
            })
            render()
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
        checkbox.setAttribute('data-form-type', 'other')
        checkbox.addEventListener('change', () => {
            filters.languages[text] = checkbox.checked;
            console.log(filters) // TODO remove
            render()
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

    var tree = {
        stats: {},
        children: {},
    }

    for (e of data) {
        path = e[I_PATH].split("\\")
        addToNode(tree, e, path)
    }

    return tree;
}

function render() {
    const start = performance.now()

    // let tree = buildTree();
    // sortTreeRecursive(tree)

    const treeBuilt = performance.now()

    html = htmlWriteNode(E("span"), g_tree)
    tree = document.getElementById("tree")
    tree.innerHTML = ""
    tree.appendChild(html)

    tree.addEventListener('toggle', (e) => {
        if (e.target.tagName === 'DETAILS') {
            collapseCache.set(e.target.id, e.target.open)
        }
        render()
    })

    const end = performance.now()
    console.log("tree build: " + (treeBuilt - start) + "ms")
    console.log("dom build: " + (end - treeBuilt) + "ms")
    console.log("total: " + (end - start) + "ms")
}

document.addEventListener("DOMContentLoaded", function() {
    // buildFilters()
    // g_tree = buildTree()
    // sortTreeRecursive(g_tree)
    // render()
})
