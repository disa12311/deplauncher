// static/app.js — simple app logic for deplauncher (with Play button + handshake)
tries++
if(tries>maxRetries){ clearInterval(iv); log('Đã hết lần thử gửi START') ; return }
sendStart()
}, 600)
}
}


// handle drag & drop
drop.addEventListener('dragover', e => { e.preventDefault(); drop.style.borderColor = '#60a5fa' })
drop.addEventListener('dragleave', e => { drop.style.borderColor = '' })
drop.addEventListener('drop', e => {
e.preventDefault(); drop.style.borderColor = ''
const f = e.dataTransfer.files && e.dataTransfer.files[0]
if(f) handleFile(f)
})


fileInput.addEventListener('change', e => {
const f = e.target.files && e.target.files[0]
if(f) handleFile(f)
})


document.getElementById('btnLoadRepo').addEventListener('click', async ()=>{
log('Fetching default Eaglercraft page from repo...')
try{
const resp = await fetch('/Eaglercraft_1.12_WASM_Offline_Download.html')
if(!resp.ok) throw new Error('404')
const blob = await resp.blob()
await saveToCache('eaglercraft_offline', blob)
player.src = URL.createObjectURL(blob)
log('Loaded from repo and cached')
}catch(err){
log('Failed to fetch from repo:', err.message)
}
})


document.getElementById('btnClearCache').addEventListener('click', async ()=>{
await clearCache()
player.src = ''
})


document.getElementById('btnOpenNewTab').addEventListener('click', async ()=>{
const blob = await getFromCache('eaglercraft_offline')
if(!blob){ alert('Không có tệp trong cache. Hãy tải trước.') ; return }
const url = URL.createObjectURL(blob)
window.open(url, '_blank')
})


btnPlay.addEventListener('click', async ()=>{
await tryPlayFromCache()
})


// listen messages from iframe (optional, trang con có thể trả lời)
window.addEventListener('message', (ev)=>{
try{
const d = ev.data
if(d && d.type === 'deplauncher:ready'){ log('Iframe báo sẵn sàng:', d) }
if(d && d.type === 'deplauncher:log'){ log('Iframe:', d.msg) }
}catch(e){/*ignore*/}
})


// initial load: try load from cache
(async ()=>{
try{
const b = await getFromCache('eaglercraft_offline')
if(b){ player.src = URL.createObjectURL(b); log('Loaded cached file') }
}catch(e){ log('Init load failed', e) }
})()


})()
