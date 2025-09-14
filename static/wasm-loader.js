// static/wasm-loader.js — helper to load a wasm Response with fallback
async function instantiateWasmResponse(response, importObject){
if(WebAssembly.instantiateStreaming){
try{
const r = await WebAssembly.instantiateStreaming(response, importObject)
return r.instance
}catch(e){
// fallthrough to arrayBuffer path
const bytes = await response.arrayBuffer()
const r = await WebAssembly.instantiate(bytes, importObject)
return r.instance
}
}else{
const bytes = await response.arrayBuffer()
const r = await WebAssembly.instantiate(bytes, importObject)
return r.instance
}
}
