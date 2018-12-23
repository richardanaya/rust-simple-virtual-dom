function getStringFromWasm(ptr, len) {
    return cachedTextDecoder.decode(getUint8Memory().subarray(ptr, ptr + len));
}

var linearMemory;

function extractStringFromMemory(offset,len){
  const stringBuffer = new Uint8Array(linearMemory.buffer, offset,len);

  // create a string from this buffer
  let str = '';
  for (let i=0; i<stringBuffer.length; i++) {
    str += String.fromCharCode(stringBuffer[i]);
  }
  return str;
}

var elementCache = [];

fetch('target/wasm32-unknown-unknown/release/simple-virtual-dom.wasm')
.then(response => response.arrayBuffer())
.then(bytes => WebAssembly.instantiate(bytes, {
  env: {
   js_log: function(start,len){
     console.log(extractStringFromMemory(start,len));
   },
   js_query_selector: function(start,len){
     var query = extractStringFromMemory(start,len);
     var el = document.querySelector(query)
     var index = elementCache.length;
     elementCache.push(el);
     return index;
   },
   js_create_element: function(start,len){
     var parentElement = elementCache[parent];
     var text = extractStringFromMemory(start,len);
     var el = document.createElement(text);
     var index = elementCache.length;
     elementCache.push(el);
     return index;
   },
   js_create_text_element: function(start,len){
     var parentElement = elementCache[parent];
     var type = extractStringFromMemory(start,len);
     var el = document.createTextNode(type);
     var index = elementCache.length;
     elementCache.push(el);
     return index;
   },
   js_append_element: function(parent,child){
     var parentElement = elementCache[parent];
     var childElement = elementCache[child];
     parentElement.append(childElement);
   },
   js_remove_child: function(parent,childIndex){
     var parentElement = elementCache[parent];
     parentElement.removeChild(
      parentElement.childNodes[childIndex]
     );
   },
   js_get_child: function(parent,childIndex){
     var parentElement = elementCache[parent];
     var el = parentElement.childNodes[childIndex];
     var index = elementCache.length;
     elementCache.push(el);
     return index;
   },
   js_replace_child: function(parent, childIndex, child){
     var parentElement = elementCache[parent];
     var childElement = elementCache[child];
     parentElement.replaceChild(childElement,parentElement.childNodes[childIndex]);
   },
   js_clear_cache: function(){
     elementCache = [];
   }
 }
}))
.then(results => {
    linearMemory = results.instance.exports.memory;
    results.instance.exports.start()
});
