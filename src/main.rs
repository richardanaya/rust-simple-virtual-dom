#![no_main]
use std::ffi::CString;
use std::os::raw::c_char;
use std::cmp;

extern {
    fn js_log(start:*mut c_char,len:usize);
    fn js_query_selector(start:*mut c_char,len:usize) -> i32;
    fn js_create_element(start:*mut c_char,len:usize) -> i32;
    fn js_create_text_element(start:*mut c_char,len:usize) -> i32;
    fn js_append_element(parent:i32,child:i32);
    fn js_remove_child(parent:i32,index:usize);
    fn js_replace_child(parent:i32, child:i32, index:usize);
    fn js_get_child(parent:i32, index:usize) ->  i32;
}

pub fn log(msg:&str) {
    let s = CString::new(msg).unwrap();
    let l = msg.len();
    unsafe {
        js_log(s.into_raw(),l);
    }
}

pub fn query_selector(msg:&str) -> i32 {
    let s = CString::new(msg).unwrap();
    let l = msg.len();
    unsafe {
        js_query_selector(s.into_raw(),l)
    }
}

fn create_element(msg:&str) -> i32{
    let s = CString::new(msg).unwrap();
    let l = msg.len();
    unsafe {
        js_create_element(s.into_raw(),l)
    }
}

fn create_text_element(msg:&str) -> i32{
    let s = CString::new(msg).unwrap();
    let l = msg.len();
    unsafe {
        js_create_text_element(s.into_raw(),l)
    }
}

fn append_element(parent:i32,child:i32) {
    if child != -1 {
        unsafe {
            js_append_element(parent,child);
        }
    }
}

fn remove_child(parent:i32,index:usize) {
    unsafe {
        js_remove_child(parent,index);
    }
}

fn replace_child(parent:i32, child:i32, index:usize) {
    unsafe {
        js_replace_child(parent,child,index);
    }
}

fn get_child(parent:i32, index:usize) -> i32 {
    unsafe {
        js_get_child(parent,index)
    }
}

struct VNode {
    node_type: String,
    children: Vec<H>
}

struct TextNode {
    text: String,
}

enum H {
    None,
    VNode(VNode),
    TextNode(TextNode),
}

fn h(node_type:&str,children:Vec<H>)->H {
    H::VNode(VNode{
        node_type: String::from(node_type),
        children: children
    })
}

fn t(text:&str)->H {
    H::TextNode(TextNode{
        text: String::from(text)
    })
}

fn create_element_from_node(node:&H) -> i32 {
    match node {
        H::VNode(vnode) => {
            let el = create_element(&vnode.node_type);
            for c in vnode.children.iter() {
                let child_element = create_element_from_node(c);
                append_element(el,child_element);
            }
            el
        },
        H::TextNode(text_node) => {
            let el = create_text_element(&text_node.text);
            el
        },
        _ => -1,
    }

}

fn update_element(parent:i32, new_node:&H, old_node:&H, index:usize){
    log(&format!("parent {}",parent));
    match old_node {
        H::None => {
            let child = create_element_from_node(&new_node);
            append_element(parent,child);
        },
        H::VNode(old_vnode)=> {
            let p2 = parent;
            log(&format!("here {} {}",parent,p2));

            match new_node {
                H::None => {
                    remove_child(parent,index)
                },
                H::VNode(new_vnode)=> {
                    if old_vnode.node_type != new_vnode.node_type {
                        let child = create_element_from_node(new_node);
                        replace_child(parent,child,index);
                    } else {
                        let new_length = new_vnode.children.len();
                        let old_length = old_vnode.children.len();
                        let min_length = cmp::min(new_length,old_length);
                        for i in 0..min_length {
                            log(&format!("same now check children {} {}",parent,i));
                            let child = get_child(parent,index);
                            update_element(
                              child,
                              &new_vnode.children[i],
                              &old_vnode.children[i],
                              i
                            );
                        }
                        if new_length > old_length {
                            let child = get_child(parent,index);
                            for i in min_length..new_length {
                                let new_child = create_element_from_node(&new_vnode.children[i]);
                                append_element(child,new_child);
                            }
                        }
                        if old_length > new_length {
                            let child = get_child(parent,index);
                            for i in min_length..old_length {
                                remove_child(child,i)
                            }
                        }
                    }
                },
                H::TextNode(_)=> {
                    let child = create_element_from_node(new_node);
                    replace_child(parent,child,index);
                }
            }
        },
        H::TextNode(old_text_node)=> {
            match new_node {
                H::None => {
                    remove_child(parent,index)
                },
                H::VNode(_)=> {
                    let child = create_element_from_node(new_node);
                    replace_child(parent,child,index);
                },
                H::TextNode(new_text_node)=> {
                    if old_text_node.text != new_text_node.text {
                        let child = create_element_from_node(new_node);
                        replace_child(parent,child,index);
                    }
                }
            }
        }
    }
}

#[no_mangle]
pub fn start() -> () {
    let body = query_selector("body");
    let mut current_vdom = H::None;

    let mut n = h("div",vec![
        h("h1",vec![t("1")]),
        h("h2",vec![t("2")]),
        h("h3",vec![t("3")])
    ]);
    update_element(body,&n,&current_vdom,0);

    current_vdom = n;
    n = h("div",vec![
        h("h3",vec![t("3")]),
        h("h2",vec![t("2")]),
        h("h1",vec![t("1")]),
        h("h5",vec![t("2")])
    ]);
    update_element(body,&n,&current_vdom,0);
}
