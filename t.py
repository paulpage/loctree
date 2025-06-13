import json
from io import StringIO

I_LANG = 0
I_PATH = 1
I_CODE = 2
I_COMMENTS = 3
I_BLANKS = 4

tree = {
    "stats": {},
    "children": {},
}

def add_to_node(node, e, path):

    node["stats"].setdefault(e[I_LANG], [0, 0, 0])
    node["stats"][e[I_LANG]][0] += e[I_CODE]
    node["stats"][e[I_LANG]][1] += e[I_COMMENTS]
    node["stats"][e[I_LANG]][2] += e[I_BLANKS]

    if len(path) > 0:
        node["children"].setdefault(path[0], {
            "stats": {},
            "children": {},
        })
        add_to_node(node["children"][path[0]], e, path[1:])



with open("list.json", "r") as f:
    data = json.load(f)

for e in data:
    path = e[I_PATH].split("\\")
    add_to_node(tree, e, path)

# with open("tree.json", "w") as f:
#     json.dump(tree, f)


def html_write_node(htmlio, node, level=0, key="node::"):
    for name, child in node['children'].items():
        code = 0
        comments = 0
        blanks = 0
        for lang, stats in child['stats'].items():
            if True:  # filters
                code += stats[0]
                comments += stats[1]
                blanks += stats[2]

        if code + comments + blanks == 0:
            continue;

        msg = f"<span><b>{name}: </b>{code} code, {comments} comments {blanks} blanks</span>"
        if child['children']:
            child_key = key + "/" + name
            is_open = 'open="true"' if level == 0 else ""
            htmlio.write(f'<details id="{child_key}" {is_open}><summary>{msg}</summary>')
            # print(len(html))
            print(htmlio.tell())
            html_write_node(htmlio, child, level + 1, child_key)
            htmlio.write('</details>')
        else:
            html.write(f'<p>{msg}</p>')
    # return html

html = StringIO()
html_write_node(html, tree)

with open("out.html", "w") as f:
    f.write(html.getvalue())
