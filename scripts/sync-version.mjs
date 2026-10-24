import { readFileSync, writeFileSync } from 'node:fs'

const pkg = JSON.parse(readFileSync('package.json', 'utf8'))
const version = pkg.version

const tauriPath = 'src-tauri/tauri.conf.json'
const tauri = JSON.parse(readFileSync(tauriPath, 'utf8'))
tauri.version = version
writeFileSync(tauriPath, `${JSON.stringify(tauri, null, 2)}\n`)

const cargoPath = 'src-tauri/Cargo.toml'
const cargo = readFileSync(cargoPath, 'utf8')
writeFileSync(cargoPath, cargo.replace(/^version = ".*"$/m, `version = "${version}"`))

console.log(`Synced version ${version} to tauri.conf.json and Cargo.toml`)
