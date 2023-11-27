## About
This loads `fontdue` as a `wasm` library and uses it to render text to a `canvas` side-by-side with the native rendering of the same character.


## 🚴 Usage

### 🛠️ Build with `wasm-pack build`

```
wasm-pack build
```

### Start a local webserver
```
cd www
npm ci
npm run start
```

(Note: `webpack` is currently dealing with an ssl issue (11/2023), and you may need to set `$env:NODE_OPTIONS = "--openssl-legacy-provider"` to work around it)

### 🔬 Test in Headless Browsers with `wasm-pack test`

```
wasm-pack test --headless --firefox
```

### Build docs folder
```
rm ./docs/*.wasm
cd www
npm ci
npm run build
```
