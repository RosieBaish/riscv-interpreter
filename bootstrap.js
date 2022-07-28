/******/ (function(modules) { // webpackBootstrap
/******/ 	// install a JSONP callback for chunk loading
/******/ 	function webpackJsonpCallback(data) {
/******/ 		var chunkIds = data[0];
/******/ 		var moreModules = data[1];
/******/
/******/
/******/ 		// add "moreModules" to the modules object,
/******/ 		// then flag all "chunkIds" as loaded and fire callback
/******/ 		var moduleId, chunkId, i = 0, resolves = [];
/******/ 		for(;i < chunkIds.length; i++) {
/******/ 			chunkId = chunkIds[i];
/******/ 			if(Object.prototype.hasOwnProperty.call(installedChunks, chunkId) && installedChunks[chunkId]) {
/******/ 				resolves.push(installedChunks[chunkId][0]);
/******/ 			}
/******/ 			installedChunks[chunkId] = 0;
/******/ 		}
/******/ 		for(moduleId in moreModules) {
/******/ 			if(Object.prototype.hasOwnProperty.call(moreModules, moduleId)) {
/******/ 				modules[moduleId] = moreModules[moduleId];
/******/ 			}
/******/ 		}
/******/ 		if(parentJsonpFunction) parentJsonpFunction(data);
/******/
/******/ 		while(resolves.length) {
/******/ 			resolves.shift()();
/******/ 		}
/******/
/******/ 	};
/******/
/******/
/******/ 	// The module cache
/******/ 	var installedModules = {};
/******/
/******/ 	// object to store loaded and loading chunks
/******/ 	// undefined = chunk not loaded, null = chunk preloaded/prefetched
/******/ 	// Promise = chunk loading, 0 = chunk loaded
/******/ 	var installedChunks = {
/******/ 		"main": 0
/******/ 	};
/******/
/******/
/******/
/******/ 	// script path function
/******/ 	function jsonpScriptSrc(chunkId) {
/******/ 		return __webpack_require__.p + "" + chunkId + ".bootstrap.js"
/******/ 	}
/******/
/******/ 	// object to store loaded and loading wasm modules
/******/ 	var installedWasmModules = {};
/******/
/******/ 	function promiseResolve() { return Promise.resolve(); }
/******/
/******/ 	var wasmImportObjects = {
/******/ 		"../pkg/riscv_interpreter_bg.wasm": function() {
/******/ 			return {
/******/ 				"./riscv_interpreter_bg.js": {
/******/ 					"__wbindgen_object_drop_ref": function(p0i32) {
/******/ 						return installedModules["../pkg/riscv_interpreter_bg.js"].exports["__wbindgen_object_drop_ref"](p0i32);
/******/ 					},
/******/ 					"__wbg_alert_cd401eff0d697978": function(p0i32,p1i32) {
/******/ 						return installedModules["../pkg/riscv_interpreter_bg.js"].exports["__wbg_alert_cd401eff0d697978"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbindgen_string_new": function(p0i32,p1i32) {
/******/ 						return installedModules["../pkg/riscv_interpreter_bg.js"].exports["__wbindgen_string_new"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_new_693216e109162396": function() {
/******/ 						return installedModules["../pkg/riscv_interpreter_bg.js"].exports["__wbg_new_693216e109162396"]();
/******/ 					},
/******/ 					"__wbg_stack_0ddaca5d1abfb52f": function(p0i32,p1i32) {
/******/ 						return installedModules["../pkg/riscv_interpreter_bg.js"].exports["__wbg_stack_0ddaca5d1abfb52f"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_error_09919627ac0992f5": function(p0i32,p1i32) {
/******/ 						return installedModules["../pkg/riscv_interpreter_bg.js"].exports["__wbg_error_09919627ac0992f5"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_instanceof_Window_42f092928baaee84": function(p0i32) {
/******/ 						return installedModules["../pkg/riscv_interpreter_bg.js"].exports["__wbg_instanceof_Window_42f092928baaee84"](p0i32);
/******/ 					},
/******/ 					"__wbg_document_15b2e504fb1556d6": function(p0i32) {
/******/ 						return installedModules["../pkg/riscv_interpreter_bg.js"].exports["__wbg_document_15b2e504fb1556d6"](p0i32);
/******/ 					},
/******/ 					"__wbg_clearInterval_a6b57bcaa4b4d5cb": function(p0i32,p1i32) {
/******/ 						return installedModules["../pkg/riscv_interpreter_bg.js"].exports["__wbg_clearInterval_a6b57bcaa4b4d5cb"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_setInterval_1931da68cc779cc0": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["../pkg/riscv_interpreter_bg.js"].exports["__wbg_setInterval_1931da68cc779cc0"](p0i32,p1i32,p2i32);
/******/ 					},
/******/ 					"__wbg_getElementById_927eae2597d26692": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["../pkg/riscv_interpreter_bg.js"].exports["__wbg_getElementById_927eae2597d26692"](p0i32,p1i32,p2i32);
/******/ 					},
/******/ 					"__wbg_getElementsByClassName_62e5b8c46eaead56": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["../pkg/riscv_interpreter_bg.js"].exports["__wbg_getElementsByClassName_62e5b8c46eaead56"](p0i32,p1i32,p2i32);
/******/ 					},
/******/ 					"__wbg_setProperty_e0774a610618c48e": function(p0i32,p1i32,p2i32,p3i32,p4i32) {
/******/ 						return installedModules["../pkg/riscv_interpreter_bg.js"].exports["__wbg_setProperty_e0774a610618c48e"](p0i32,p1i32,p2i32,p3i32,p4i32);
/******/ 					},
/******/ 					"__wbg_length_7399238b14adcf66": function(p0i32) {
/******/ 						return installedModules["../pkg/riscv_interpreter_bg.js"].exports["__wbg_length_7399238b14adcf66"](p0i32);
/******/ 					},
/******/ 					"__wbg_item_4175e9d985466d3c": function(p0i32,p1i32) {
/******/ 						return installedModules["../pkg/riscv_interpreter_bg.js"].exports["__wbg_item_4175e9d985466d3c"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_classList_965edecde8dc2a79": function(p0i32) {
/******/ 						return installedModules["../pkg/riscv_interpreter_bg.js"].exports["__wbg_classList_965edecde8dc2a79"](p0i32);
/******/ 					},
/******/ 					"__wbg_setinnerHTML_fe7eeed1b320a302": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["../pkg/riscv_interpreter_bg.js"].exports["__wbg_setinnerHTML_fe7eeed1b320a302"](p0i32,p1i32,p2i32);
/******/ 					},
/******/ 					"__wbg_children_24719c7bfdf8fff0": function(p0i32) {
/******/ 						return installedModules["../pkg/riscv_interpreter_bg.js"].exports["__wbg_children_24719c7bfdf8fff0"](p0i32);
/******/ 					},
/******/ 					"__wbg_getElementsByClassName_89215da102deee39": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["../pkg/riscv_interpreter_bg.js"].exports["__wbg_getElementsByClassName_89215da102deee39"](p0i32,p1i32,p2i32);
/******/ 					},
/******/ 					"__wbg_log_17733ab6fa45831d": function(p0i32) {
/******/ 						return installedModules["../pkg/riscv_interpreter_bg.js"].exports["__wbg_log_17733ab6fa45831d"](p0i32);
/******/ 					},
/******/ 					"__wbg_instanceof_HtmlElement_057bfd3477e9b9b6": function(p0i32) {
/******/ 						return installedModules["../pkg/riscv_interpreter_bg.js"].exports["__wbg_instanceof_HtmlElement_057bfd3477e9b9b6"](p0i32);
/******/ 					},
/******/ 					"__wbg_style_365767989176e8d2": function(p0i32) {
/******/ 						return installedModules["../pkg/riscv_interpreter_bg.js"].exports["__wbg_style_365767989176e8d2"](p0i32);
/******/ 					},
/******/ 					"__wbg_instanceof_HtmlInputElement_3fad42774bc62388": function(p0i32) {
/******/ 						return installedModules["../pkg/riscv_interpreter_bg.js"].exports["__wbg_instanceof_HtmlInputElement_3fad42774bc62388"](p0i32);
/******/ 					},
/******/ 					"__wbg_value_30770021ca38e0db": function(p0i32,p1i32) {
/******/ 						return installedModules["../pkg/riscv_interpreter_bg.js"].exports["__wbg_value_30770021ca38e0db"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_instanceof_HtmlTextAreaElement_f8666dc47678e5c0": function(p0i32) {
/******/ 						return installedModules["../pkg/riscv_interpreter_bg.js"].exports["__wbg_instanceof_HtmlTextAreaElement_f8666dc47678e5c0"](p0i32);
/******/ 					},
/******/ 					"__wbg_value_eb32f706ae6bfab2": function(p0i32,p1i32) {
/******/ 						return installedModules["../pkg/riscv_interpreter_bg.js"].exports["__wbg_value_eb32f706ae6bfab2"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_add_a1fa1336c6b306df": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["../pkg/riscv_interpreter_bg.js"].exports["__wbg_add_a1fa1336c6b306df"](p0i32,p1i32,p2i32);
/******/ 					},
/******/ 					"__wbg_contains_b76df5a41fa270ed": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["../pkg/riscv_interpreter_bg.js"].exports["__wbg_contains_b76df5a41fa270ed"](p0i32,p1i32,p2i32);
/******/ 					},
/******/ 					"__wbg_remove_dce5eca3c9fcea70": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["../pkg/riscv_interpreter_bg.js"].exports["__wbg_remove_dce5eca3c9fcea70"](p0i32,p1i32,p2i32);
/******/ 					},
/******/ 					"__wbg_parentElement_14138ef2ff0b9c88": function(p0i32) {
/******/ 						return installedModules["../pkg/riscv_interpreter_bg.js"].exports["__wbg_parentElement_14138ef2ff0b9c88"](p0i32);
/******/ 					},
/******/ 					"__wbg_newnoargs_971e9a5abe185139": function(p0i32,p1i32) {
/******/ 						return installedModules["../pkg/riscv_interpreter_bg.js"].exports["__wbg_newnoargs_971e9a5abe185139"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_call_33d7bcddbbfa394a": function(p0i32,p1i32) {
/******/ 						return installedModules["../pkg/riscv_interpreter_bg.js"].exports["__wbg_call_33d7bcddbbfa394a"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_self_fd00a1ef86d1b2ed": function() {
/******/ 						return installedModules["../pkg/riscv_interpreter_bg.js"].exports["__wbg_self_fd00a1ef86d1b2ed"]();
/******/ 					},
/******/ 					"__wbg_window_6f6e346d8bbd61d7": function() {
/******/ 						return installedModules["../pkg/riscv_interpreter_bg.js"].exports["__wbg_window_6f6e346d8bbd61d7"]();
/******/ 					},
/******/ 					"__wbg_globalThis_3348936ac49df00a": function() {
/******/ 						return installedModules["../pkg/riscv_interpreter_bg.js"].exports["__wbg_globalThis_3348936ac49df00a"]();
/******/ 					},
/******/ 					"__wbg_global_67175caf56f55ca9": function() {
/******/ 						return installedModules["../pkg/riscv_interpreter_bg.js"].exports["__wbg_global_67175caf56f55ca9"]();
/******/ 					},
/******/ 					"__wbindgen_is_undefined": function(p0i32) {
/******/ 						return installedModules["../pkg/riscv_interpreter_bg.js"].exports["__wbindgen_is_undefined"](p0i32);
/******/ 					},
/******/ 					"__wbindgen_object_clone_ref": function(p0i32) {
/******/ 						return installedModules["../pkg/riscv_interpreter_bg.js"].exports["__wbindgen_object_clone_ref"](p0i32);
/******/ 					},
/******/ 					"__wbindgen_debug_string": function(p0i32,p1i32) {
/******/ 						return installedModules["../pkg/riscv_interpreter_bg.js"].exports["__wbindgen_debug_string"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbindgen_throw": function(p0i32,p1i32) {
/******/ 						return installedModules["../pkg/riscv_interpreter_bg.js"].exports["__wbindgen_throw"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbindgen_closure_wrapper461": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["../pkg/riscv_interpreter_bg.js"].exports["__wbindgen_closure_wrapper461"](p0i32,p1i32,p2i32);
/******/ 					}
/******/ 				}
/******/ 			};
/******/ 		},
/******/ 	};
/******/
/******/ 	// The require function
/******/ 	function __webpack_require__(moduleId) {
/******/
/******/ 		// Check if module is in cache
/******/ 		if(installedModules[moduleId]) {
/******/ 			return installedModules[moduleId].exports;
/******/ 		}
/******/ 		// Create a new module (and put it into the cache)
/******/ 		var module = installedModules[moduleId] = {
/******/ 			i: moduleId,
/******/ 			l: false,
/******/ 			exports: {}
/******/ 		};
/******/
/******/ 		// Execute the module function
/******/ 		modules[moduleId].call(module.exports, module, module.exports, __webpack_require__);
/******/
/******/ 		// Flag the module as loaded
/******/ 		module.l = true;
/******/
/******/ 		// Return the exports of the module
/******/ 		return module.exports;
/******/ 	}
/******/
/******/ 	// This file contains only the entry chunk.
/******/ 	// The chunk loading function for additional chunks
/******/ 	__webpack_require__.e = function requireEnsure(chunkId) {
/******/ 		var promises = [];
/******/
/******/
/******/ 		// JSONP chunk loading for javascript
/******/
/******/ 		var installedChunkData = installedChunks[chunkId];
/******/ 		if(installedChunkData !== 0) { // 0 means "already installed".
/******/
/******/ 			// a Promise means "currently loading".
/******/ 			if(installedChunkData) {
/******/ 				promises.push(installedChunkData[2]);
/******/ 			} else {
/******/ 				// setup Promise in chunk cache
/******/ 				var promise = new Promise(function(resolve, reject) {
/******/ 					installedChunkData = installedChunks[chunkId] = [resolve, reject];
/******/ 				});
/******/ 				promises.push(installedChunkData[2] = promise);
/******/
/******/ 				// start chunk loading
/******/ 				var script = document.createElement('script');
/******/ 				var onScriptComplete;
/******/
/******/ 				script.charset = 'utf-8';
/******/ 				script.timeout = 120;
/******/ 				if (__webpack_require__.nc) {
/******/ 					script.setAttribute("nonce", __webpack_require__.nc);
/******/ 				}
/******/ 				script.src = jsonpScriptSrc(chunkId);
/******/
/******/ 				// create error before stack unwound to get useful stacktrace later
/******/ 				var error = new Error();
/******/ 				onScriptComplete = function (event) {
/******/ 					// avoid mem leaks in IE.
/******/ 					script.onerror = script.onload = null;
/******/ 					clearTimeout(timeout);
/******/ 					var chunk = installedChunks[chunkId];
/******/ 					if(chunk !== 0) {
/******/ 						if(chunk) {
/******/ 							var errorType = event && (event.type === 'load' ? 'missing' : event.type);
/******/ 							var realSrc = event && event.target && event.target.src;
/******/ 							error.message = 'Loading chunk ' + chunkId + ' failed.\n(' + errorType + ': ' + realSrc + ')';
/******/ 							error.name = 'ChunkLoadError';
/******/ 							error.type = errorType;
/******/ 							error.request = realSrc;
/******/ 							chunk[1](error);
/******/ 						}
/******/ 						installedChunks[chunkId] = undefined;
/******/ 					}
/******/ 				};
/******/ 				var timeout = setTimeout(function(){
/******/ 					onScriptComplete({ type: 'timeout', target: script });
/******/ 				}, 120000);
/******/ 				script.onerror = script.onload = onScriptComplete;
/******/ 				document.head.appendChild(script);
/******/ 			}
/******/ 		}
/******/
/******/ 		// Fetch + compile chunk loading for webassembly
/******/
/******/ 		var wasmModules = {"0":["../pkg/riscv_interpreter_bg.wasm"]}[chunkId] || [];
/******/
/******/ 		wasmModules.forEach(function(wasmModuleId) {
/******/ 			var installedWasmModuleData = installedWasmModules[wasmModuleId];
/******/
/******/ 			// a Promise means "currently loading" or "already loaded".
/******/ 			if(installedWasmModuleData)
/******/ 				promises.push(installedWasmModuleData);
/******/ 			else {
/******/ 				var importObject = wasmImportObjects[wasmModuleId]();
/******/ 				var req = fetch(__webpack_require__.p + "" + {"../pkg/riscv_interpreter_bg.wasm":"bb02659ad343260bd639"}[wasmModuleId] + ".module.wasm");
/******/ 				var promise;
/******/ 				if(importObject instanceof Promise && typeof WebAssembly.compileStreaming === 'function') {
/******/ 					promise = Promise.all([WebAssembly.compileStreaming(req), importObject]).then(function(items) {
/******/ 						return WebAssembly.instantiate(items[0], items[1]);
/******/ 					});
/******/ 				} else if(typeof WebAssembly.instantiateStreaming === 'function') {
/******/ 					promise = WebAssembly.instantiateStreaming(req, importObject);
/******/ 				} else {
/******/ 					var bytesPromise = req.then(function(x) { return x.arrayBuffer(); });
/******/ 					promise = bytesPromise.then(function(bytes) {
/******/ 						return WebAssembly.instantiate(bytes, importObject);
/******/ 					});
/******/ 				}
/******/ 				promises.push(installedWasmModules[wasmModuleId] = promise.then(function(res) {
/******/ 					return __webpack_require__.w[wasmModuleId] = (res.instance || res).exports;
/******/ 				}));
/******/ 			}
/******/ 		});
/******/ 		return Promise.all(promises);
/******/ 	};
/******/
/******/ 	// expose the modules object (__webpack_modules__)
/******/ 	__webpack_require__.m = modules;
/******/
/******/ 	// expose the module cache
/******/ 	__webpack_require__.c = installedModules;
/******/
/******/ 	// define getter function for harmony exports
/******/ 	__webpack_require__.d = function(exports, name, getter) {
/******/ 		if(!__webpack_require__.o(exports, name)) {
/******/ 			Object.defineProperty(exports, name, { enumerable: true, get: getter });
/******/ 		}
/******/ 	};
/******/
/******/ 	// define __esModule on exports
/******/ 	__webpack_require__.r = function(exports) {
/******/ 		if(typeof Symbol !== 'undefined' && Symbol.toStringTag) {
/******/ 			Object.defineProperty(exports, Symbol.toStringTag, { value: 'Module' });
/******/ 		}
/******/ 		Object.defineProperty(exports, '__esModule', { value: true });
/******/ 	};
/******/
/******/ 	// create a fake namespace object
/******/ 	// mode & 1: value is a module id, require it
/******/ 	// mode & 2: merge all properties of value into the ns
/******/ 	// mode & 4: return value when already ns object
/******/ 	// mode & 8|1: behave like require
/******/ 	__webpack_require__.t = function(value, mode) {
/******/ 		if(mode & 1) value = __webpack_require__(value);
/******/ 		if(mode & 8) return value;
/******/ 		if((mode & 4) && typeof value === 'object' && value && value.__esModule) return value;
/******/ 		var ns = Object.create(null);
/******/ 		__webpack_require__.r(ns);
/******/ 		Object.defineProperty(ns, 'default', { enumerable: true, value: value });
/******/ 		if(mode & 2 && typeof value != 'string') for(var key in value) __webpack_require__.d(ns, key, function(key) { return value[key]; }.bind(null, key));
/******/ 		return ns;
/******/ 	};
/******/
/******/ 	// getDefaultExport function for compatibility with non-harmony modules
/******/ 	__webpack_require__.n = function(module) {
/******/ 		var getter = module && module.__esModule ?
/******/ 			function getDefault() { return module['default']; } :
/******/ 			function getModuleExports() { return module; };
/******/ 		__webpack_require__.d(getter, 'a', getter);
/******/ 		return getter;
/******/ 	};
/******/
/******/ 	// Object.prototype.hasOwnProperty.call
/******/ 	__webpack_require__.o = function(object, property) { return Object.prototype.hasOwnProperty.call(object, property); };
/******/
/******/ 	// __webpack_public_path__
/******/ 	__webpack_require__.p = "";
/******/
/******/ 	// on error function for async loading
/******/ 	__webpack_require__.oe = function(err) { console.error(err); throw err; };
/******/
/******/ 	// object with all WebAssembly.instance exports
/******/ 	__webpack_require__.w = {};
/******/
/******/ 	var jsonpArray = window["webpackJsonp"] = window["webpackJsonp"] || [];
/******/ 	var oldJsonpFunction = jsonpArray.push.bind(jsonpArray);
/******/ 	jsonpArray.push = webpackJsonpCallback;
/******/ 	jsonpArray = jsonpArray.slice();
/******/ 	for(var i = 0; i < jsonpArray.length; i++) webpackJsonpCallback(jsonpArray[i]);
/******/ 	var parentJsonpFunction = oldJsonpFunction;
/******/
/******/
/******/ 	// Load entry module and return exports
/******/ 	return __webpack_require__(__webpack_require__.s = "./bootstrap.js");
/******/ })
/************************************************************************/
/******/ ({

/***/ "./bootstrap.js":
/*!**********************!*\
  !*** ./bootstrap.js ***!
  \**********************/
/*! no static exports found */
/***/ (function(module, exports, __webpack_require__) {

eval("// A dependency graph that contains any wasm must all be imported\n// asynchronously. This `bootstrap.js` file does the single async import, so\n// that no one else needs to worry about it again.\n__webpack_require__.e(/*! import() */ 0).then(__webpack_require__.bind(null, /*! ./index.js */ \"./index.js\"))\n  .catch(e => console.error(\"Error importing `index.js`:\", e));\n\n\n//# sourceURL=webpack:///./bootstrap.js?");

/***/ })

/******/ });