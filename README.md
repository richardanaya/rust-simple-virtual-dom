# Simple Virtual DOM in Rust + WASM

This project is my attempt as simply implementing a very basic virtual DOM from scratch. There's a few interesting challenges in talking with javascript as well as the algorithm itself.

Let's talk first about the challenges of WASM interacting with DOM. Since web assembly doesn't have any API for interacting with the DOM, we must interact with the DOM through Javascript. There is however an additional difficulty in that WASM-JS communication can only be done through simple number types (integers and floats). This brings the first question of how do we pass a string from WASM to javascript?

Our one saving grace is that javascript has access to the memory of our WASM application. So instantiating a string in WASM can be viewed in JS if we know two things:

1) the start of the string
2) the length of the string

In this project, it might be easiest to see how this is done by looking at the `log` function in WASM. We have a helper function `log` that calls a javascript function `js_log`. `log` creates a C string type, and we can get a pointer which represents the memory location to send to javascript. Javascript can then iterate through those characters and form a string of it's own to do some action with (see `extractStringFromMemory` in `index.js`).

We'll be using multiple functions that pass along strings to javascript to perform various DOM manipulation, so whenever you see a start and length its talking about a string being sent over the WASM-JS bridge.

Example:

```rust
extern {
    fn js_log(start:*mut c_char,len:usize);
    ...
}

pub fn log(msg:&str) {
    let s = CString::new(msg).unwrap();
    let l = msg.len();
    unsafe {
        js_log(s.into_raw(),l);
    }
}
```

# DOM management

Since WASM can't pass around DOM elements directly, what we need is some sort of system for talking about the DOM we are going to operate on. In this project, whenever DOM is queried or created, we give that piece of DOM an integer ID.

For instance, if we queried the `body` element, we assign that element a number and store that in an dictionary `number -> Element`. Let's assum the number we get for referring to the `body` is 123.  Now whenever we perform DOM operations on the body, say, setting the innerHTML. We can simply call `set_inner_html(123,"hello!")`.

# Virtual DOM

There is a great (but incomplete article) https://medium.com/@deathmood/how-to-write-your-own-virtual-dom-ee74acc13060 that describe the process of creating a Virtual DOM from scratch.

The important thing to remember is that we are trying to do as minimal DOM operations as possible. Manipulating the DOM is incredibly expensive, so if we can find any way of avoiding interactions with it the better.  How virtual DOM accomplishes this is by representing our DOM as a tree of nodes. Then each time we render, we compare the tree of nodes we currently have to the new tree of nodes, and we can determine what real DOM needs to be created,removed, replaced, or modified.

In this example i'm making a pretty massive simplification: **this is a virtual DOM for elements with NO attributes or event handlers**

This simplification makes it alot easier to see the basic operations going on. In Rust we represent VirtualDom as follows.

```Rust
// ElementNode represents an html element
struct ElementNode {
    node_type: String,
    children: Vec<VirtualDomNode>
}

// TextNode represents text that is mixed in with elements
struct TextNode {
    text: String,
}

// We use an enumeration to represent these two
// plus an empty DOM node to represent nothing
enum VirtualDomNode {
    None,
    ElementNode(ElementNode),
    TextNode(TextNode),
}

// VirtualDom represents a virtual dom tree
struct VirtualDom {
    root_node:VirtualDomNode
}

impl VirtualDom {
    // new creates an empty VirtualDom
    fn new() -> VirtualDom {
        VirtualDom {
            root_node: VirtualDomNode::None
        }
    }

    // Compares two virtual dom tree structures and updates the real DOM 
    // then stores the new dom tree for future comparisons
    fn render(&mut self, el:i32, new_vdom:VirtualDomNode){
        // This is where the magic happens
        update_element(el,&new_vdom,&self.root_node,0);
        self.root_node = new_vdom;
    }
}
```

For a simple html:

```html
<div>
    <h1>hello!</h1>
</div>
```

A simple tree of DOM might be represented thus as:

```rust
VirtualDomNode::ElementNode(ElementNode{
    node_type: String::from("div"),
    children: vec![
        VirtualDomNode::ElementNode(ElementNode{
            node_type: String::from("h1"),
            children: vec![
                VirtualDomNode::TextNode(TextNode{
                    text: String::from("hello"),
                })
            ]
        }
    ]
})
```

This is a little verbose though, we we have two helper functions:

```rust
fn h(node_type:&str,children:Vec<VirtualDomNode>)->VirtualDomNode {
    VirtualDomNode::ElementNode(ElementNode{
        node_type: String::from(node_type),
        children: children
    })
}

fn t(text:&str)->VirtualDomNode {
    VirtualDomNode::TextNode(TextNode{
        text: String::from(text)
    })
}
```

So we can easily represent virtual DOM

```rust
h("div",vec![
    h("h1",vec![
        t("hello!")
    ])
])
```

This allows us to easily interact with our virtual DOM:

```rust
// Let's get a handle to our body element
let body = query_selector("body");

// Let's create our empty virtual dom
let mut vd = VirtualDom::new();

// Render a simple list to the body element
vd.render(body, h("div",vec![
    h("h1",vec![t("1")]),
    h("h2",vec![t("2")]),
    h("h3",vec![t("3")])
]));

// Render a new virtual dom tree to the body element that is the reverseof the list
vd.render(body, h("div",vec![
    h("h1",vec![t("3")]),
    h("h2",vec![t("2")]),
    h("h3",vec![t("1")])
]))
// ONLY h1 and h3's text node should change
```

Let's consider what happens on the first rendering.  We have a virtual dom tree with an `None` node in it, and some new virtual dom tree coming in that has elements and text. Our tree comparison is simple in this first rendering since we only have all new nodes we need to create real DOM elements for. So let's look how we might create that tree of real DOM. We have three scenerios to handle:

```rust
fn create_element_from_node(node:&VirtualDomNode) -> i32 {
    match node {
        VirtualDomNode::ElementNode(vnode) => {
            let el = create_element(&vnode.node_type);
            // Recursively create child nodes as well
            for c in vnode.children.iter() {
                let child_element = create_element_from_node(c);
                append_element(el,child_element);
            }
            el
        },
        VirtualDomNode::TextNode(text_node) => {
            let el = create_text_element(&text_node.text);
            el
        },
        VirtualDomNode::None => {
            let el = create_text_element("");
            el
        }
    }

}
```

Finally once we create this tree of nodes, we simply attach the top most element to the body element.
