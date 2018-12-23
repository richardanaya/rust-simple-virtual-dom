# Simple Virtual DOM in Rust + WASM

This project is my attempt as simply implementing a very basic virtual DOM from scratch. There's a few interesting challenges in talking with javascript as well as the algorithm itself.

Let's talk first about the challenges of WASM interacting with DOM. Since web assembly doesn't have any API for interacting with the DOM, we must interact with the DOM through Javascript. There is however an additional difficulty in that WASM-JS communication can only be done through simple number types (integers and floats). This brings the first question of how do we pass a string from WASM to javascript?

Our one saving grace is that javascript has access to the memory of our WASM application. So instantiating a string in WASM can be viewed in JS if we know two things:

1) the start of the string
2) the length of the string

In this project, it might be easiest to see how this is done by looking at the `log` function in WASM. We have a helper function `log` that calls a javascript function `js_log`. `log` creates a C string type, and we can get a pointer which represents the memory location to send to javascript.

We'll be using multiple functions that pass along strings to javascript to perform various DOM manipulation, so whenever you see a start and length its talking about a string.

# DOM management

Since WASM can't pass around DOM elements directly, what we need is some sort of system for talking about the DOM we are going to operate on. In this project, whenever DOM is queried or created, we give that piece of DOM an integer ID.

For instance, if we queried the `body` tag, we assign that tag a number and store that in an dictionary `number -> Element`. Let's assum the number we get for referring to the `body` is 123.  Now whenever we perform DOM operations on the body, say, setting the innerHTML. We can simply call `set_inner_html(123,"hello!")`.

# Virtual DOM

There is a great (but incomplete article) https://medium.com/@deathmood/how-to-write-your-own-virtual-dom-ee74acc13060 that describe the process of creating a Virtual DOM from scratch.

The important thing to remember is that we are trying to do as minimal DOM operations as possible. Manipulating the DOM is incredibly expensive, so if we can find any way of avoiding interactions with it the better.  How virtual DOM accomplishes this is by representing our DOM as a tree of nodes. Then each time we render, we compare the tree of nodes we currently have to the new tree of nodes, and we can determine what real DOM needs to be created,removed, replaced, or modified.

In this example i'm making a pretty massive simplification: *this is a virtual DOM for elements with NO attributes or event handlers*

This simplification makes it alot easier to see the basic operations going on.
