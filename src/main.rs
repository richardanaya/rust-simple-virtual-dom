#![no_main]
use std::cmp;
use std::ffi::CString;
use std::os::raw::c_char;

// We're going to need a number of helper functions to talk to javascript
// so we can create, remove, modify our elements. Since in WASM you can
// only pass around numbers, some of these functions hand off memory
// positions of strings.
// Also, since we can't pass around elements, was pass around int32 handles
// to DOM elements that exist in javascript. Look for a variable named
// elementCache.

extern "C" {
    fn js_log(start: *mut c_char, len: usize);
    fn js_query_selector(start: *mut c_char, len: usize) -> i32;
    fn js_create_element(start: *mut c_char, len: usize) -> i32;
    fn js_create_text_element(start: *mut c_char, len: usize) -> i32;
    fn js_append_element(parent: i32, child: i32);
    fn js_remove_child(parent: i32, child_index: usize);
    fn js_replace_child(parent: i32, child_index: usize, child: i32);
    fn js_get_child(parent: i32, child_index: usize) -> i32;
}

pub fn log(msg: &str) {
    let s = CString::new(msg).unwrap();
    let l = msg.len();
    unsafe {
        js_log(s.into_raw(), l);
    }
}

pub fn query_selector(msg: &str) -> i32 {
    let s = CString::new(msg).unwrap();
    let l = msg.len();
    unsafe { js_query_selector(s.into_raw(), l) }
}

fn create_element(msg: &str) -> i32 {
    let s = CString::new(msg).unwrap();
    let l = msg.len();
    unsafe { js_create_element(s.into_raw(), l) }
}

fn create_text_element(msg: &str) -> i32 {
    let s = CString::new(msg).unwrap();
    let l = msg.len();
    unsafe { js_create_text_element(s.into_raw(), l) }
}

fn append_element(parent: i32, child: i32) {
    unsafe {
        js_append_element(parent, child);
    }
}

fn remove_child(parent: i32, child_index: usize) {
    unsafe {
        js_remove_child(parent, child_index);
    }
}

fn replace_child(parent: i32, child_index: usize, child: i32) {
    unsafe {
        js_replace_child(parent, child_index, child);
    }
}

fn get_child(parent: i32, child_index: usize) -> i32 {
    unsafe { js_get_child(parent, child_index) }
}

// A virtual dom tree is comprised of two types of nodes

// ElementNode represents a DOM element (div, h1, etc.)
struct ElementNode {
    node_type: String,
    children: Vec<VirtualDomNode>,
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

// These are helper functions to create virtual dom nodes
fn h(node_type: &str, children: Vec<VirtualDomNode>) -> VirtualDomNode {
    VirtualDomNode::ElementNode(ElementNode {
        node_type: String::from(node_type),
        children: children,
    })
}

fn t(text: &str) -> VirtualDomNode {
    VirtualDomNode::TextNode(TextNode {
        text: String::from(text),
    })
}

// create_element_from_node is a helper for creating real DOM from virtual DOM
fn create_element_from_node(node: &VirtualDomNode) -> i32 {
    match node {
        VirtualDomNode::ElementNode(vnode) => {
            let el = create_element(&vnode.node_type);
            for c in vnode.children.iter() {
                let child_element = create_element_from_node(c);
                append_element(el, child_element);
            }
            el
        }
        VirtualDomNode::TextNode(text_node) => {
            let el = create_text_element(&text_node.text);
            el
        }
        VirtualDomNode::None => {
            let el = create_text_element("");
            el
        }
    }
}

fn update_element(
    parent: i32,
    child_index: usize,
    new_node: &VirtualDomNode,
    old_node: &VirtualDomNode,
) {
    match old_node {
        VirtualDomNode::None => {
            let child = create_element_from_node(&new_node);
            append_element(parent, child);
        }
        VirtualDomNode::ElementNode(old_vnode) => match new_node {
            VirtualDomNode::None => remove_child(parent, child_index),
            VirtualDomNode::ElementNode(new_vnode) => {
                if old_vnode.node_type != new_vnode.node_type {
                    let child = create_element_from_node(new_node);
                    replace_child(parent, child_index, child);
                } else {
                    let new_length = new_vnode.children.len();
                    let old_length = old_vnode.children.len();
                    let min_length = cmp::min(new_length, old_length);
                    for i in 0..min_length {
                        let child = get_child(parent, child_index);
                        update_element(child, i, &new_vnode.children[i], &old_vnode.children[i]);
                    }
                    if new_length > old_length {
                        let child = get_child(parent, child_index);
                        for i in min_length..new_length {
                            let new_child = create_element_from_node(&new_vnode.children[i]);
                            append_element(child, new_child);
                        }
                    }
                    if old_length > new_length {
                        let child = get_child(parent, child_index);
                        for i in min_length..old_length {
                            remove_child(child, i)
                        }
                    }
                }
            }
            VirtualDomNode::TextNode(_) => {
                let child = create_element_from_node(new_node);
                replace_child(parent, child_index, child);
            }
        },
        VirtualDomNode::TextNode(old_text_node) => match new_node {
            VirtualDomNode::None => remove_child(parent, child_index),
            VirtualDomNode::ElementNode(_) => {
                let child = create_element_from_node(new_node);
                replace_child(parent, child_index, child);
            }
            VirtualDomNode::TextNode(new_text_node) => {
                if old_text_node.text != new_text_node.text {
                    let child = create_element_from_node(new_node);
                    replace_child(parent, child_index, child);
                }
            }
        },
    }
}

struct VirtualDom {
    root_node: VirtualDomNode,
}

impl VirtualDom {
    fn new() -> VirtualDom {
        VirtualDom {
            root_node: VirtualDomNode::None,
        }
    }

    fn render(&mut self, el: i32, new_vdom: VirtualDomNode) {
        update_element(el, 0, &new_vdom, &self.root_node);
        self.root_node = new_vdom;
    }
}

#[no_mangle]
pub fn start() -> () {
    // Let's get a handle to our body element
    let body = query_selector("body");

    // Let's create our empty virtual dom
    let mut vd = VirtualDom::new();

    // Render a simple list to the body element
    vd.render(
        body,
        h(
            "div",
            vec![
                h("h1", vec![t("1")]),
                h("h2", vec![t("2")]),
                h("h3", vec![t("3")]),
            ],
        ),
    );

    // Render a new virtual dom tree to the body element
    vd.render(
        body,
        h(
            "div",
            vec![
                h("h1", vec![t("3")]),
                h("h2", vec![t("2")]),
                h("h3", vec![t("1")]),
            ],
        ),
    )
}
