/*!
  Highlight.js v11.3.1 (git: 2a972d8658)
  (c) 2006-2023 Ivan Sagalaev and other contributors
  License: BSD-3-Clause
 */
var hljs=function(){"use strict";var e={exports:{}};function t(e){
return e instanceof Map?e.clear=e.delete=e.set=()=>{
throw Error("map is read-only")}:e instanceof Set&&(e.add=e.clear=e.delete=()=>{
throw Error("set is read-only")
}),Object.freeze(e),Object.getOwnPropertyNames(e).forEach((n=>{var s=e[n]
;"object"!=typeof s||Object.isFrozen(s)||t(s)})),e}
e.exports=t,e.exports.default=t;var n=e.exports;class s{constructor(e){
void 0===e.data&&(e.data={}),this.data=e.data,this.isMatchIgnored=!1}
ignoreMatch(){this.isMatchIgnored=!0}}function i(e){
return e.replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;").replace(/'/g,"&#x27;")
}function a(e,...t){const n=Object.create(null);for(const t in e)n[t]=e[t]
;return t.forEach((e=>{for(const t in e)n[t]=e[t]})),n}const r=e=>!!e.kind
;class o{constructor(e,t){
this.buffer="",this.classPrefix=t.classPrefix,e.walk(this)}addText(e){
this.buffer+=i(e)}openNode(e){if(!r(e))return;let t=e.kind
;t=e.sublanguage?"language-"+t:((e,{prefix:t})=>{if(e.includes(".")){
const n=e.split(".")
;return[`${t}${n.shift()}`,...n.map(((e,t)=>`${e}${"_".repeat(t+1)}`))].join(" ")
}return`${t}${e}`})(t,{prefix:this.classPrefix}),this.span(t)}closeNode(e){
r(e)&&(this.buffer+="</span>")}value(){return this.buffer}span(e){
this.buffer+=`<span class="${e}">`}}class l{constructor(){this.rootNode={
children:[]},this.stack=[this.rootNode]}get top(){
return this.stack[this.stack.length-1]}get root(){return this.rootNode}add(e){
this.top.children.push(e)}openNode(e){const t={kind:e,children:[]}
;this.add(t),this.stack.push(t)}closeNode(){
if(this.stack.length>1)return this.stack.pop()}closeAllNodes(){
for(;this.closeNode(););}toJSON(){return JSON.stringify(this.rootNode,null,4)}
walk(e){return this.constructor._walk(e,this.rootNode)}static _walk(e,t){
return"string"==typeof t?e.addText(t):t.children&&(e.openNode(t),
t.children.forEach((t=>this._walk(e,t))),e.closeNode(t)),e}static _collapse(e){
"string"!=typeof e&&e.children&&(e.children.every((e=>"string"==typeof e))?e.children=[e.children.join("")]:e.children.forEach((e=>{
l._collapse(e)})))}}class c extends l{constructor(e){super(),this.options=e}
addKeyword(e,t){""!==e&&(this.openNode(t),this.addText(e),this.closeNode())}
addText(e){""!==e&&this.add(e)}addSublanguage(e,t){const n=e.root
;n.kind=t,n.sublanguage=!0,this.add(n)}toHTML(){
return new o(this,this.options).value()}finalize(){return!0}}function d(e){
return e?"string"==typeof e?e:e.source:null}function u(e){return b("(?=",e,")")}
function g(e){return b("(?:",e,")*")}function h(e){return b("(?:",e,")?")}
function b(...e){return e.map((e=>d(e))).join("")}function p(...e){const t=(e=>{
const t=e[e.length-1]
;return"object"==typeof t&&t.constructor===Object?(e.splice(e.length-1,1),t):{}
})(e);return"("+(t.capture?"":"?:")+e.map((e=>d(e))).join("|")+")"}
function f(e){return RegExp(e.toString()+"|").exec("").length-1}
const m=/\[(?:[^\\\]]|\\.)*\]|\(\??|\\([1-9][0-9]*)|\\./
;function E(e,{joinWith:t}){let n=0;return e.map((e=>{n+=1;const t=n
;let s=d(e),i="";for(;s.length>0;){const e=m.exec(s);if(!e){i+=s;break}
i+=s.substring(0,e.index),
s=s.substring(e.index+e[0].length),"\\"===e[0][0]&&e[1]?i+="\\"+(Number(e[1])+t):(i+=e[0],
"("===e[0]&&n++)}return i})).map((e=>`(${e})`)).join(t)}
const _="[a-zA-Z]\\w*",w="[a-zA-Z_]\\w*",y="\\b\\d+(\\.\\d+)?",N="(-?)(\\b0[xX][a-fA-F0-9]+|(\\b\\d+(\\.\\d*)?|\\.\\d+)([eE][-+]?\\d+)?)",x="\\b(0b[01]+)",k={
begin:"\\\\[\\s\\S]",relevance:0},v={scope:"string",begin:"'",end:"'",
illegal:"\\n",contains:[k]},O={scope:"string",begin:'"',end:'"',illegal:"\\n",
contains:[k]},S=(e,t,n={})=>{const s=a({scope:"comment",begin:e,end:t,
contains:[]},n);s.contains.push({scope:"doctag",
begin:"[ ]*(?=(TODO|FIXME|NOTE|BUG|OPTIMIZE|HACK|XXX):)",
end:/(TODO|FIXME|NOTE|BUG|OPTIMIZE|HACK|XXX):/,excludeBegin:!0,relevance:0})
;const i=p("I","a","is","so","us","to","at","if","in","it","on",/[A-Za-z]+['](d|ve|re|ll|t|s|n)/,/[A-Za-z]+[-][a-z]+/,/[A-Za-z][a-z]{2,}/)
;return s.contains.push({begin:b(/[ ]+/,"(",i,/[.]?[:]?([.][ ]|[ ])/,"){3}")}),s
},R=S("//","$"),M=S("/\\*","\\*/"),I=S("#","$");var A=Object.freeze({
__proto__:null,MATCH_NOTHING_RE:/\b\B/,IDENT_RE:_,UNDERSCORE_IDENT_RE:w,
NUMBER_RE:y,C_NUMBER_RE:N,BINARY_NUMBER_RE:x,
RE_STARTERS_RE:"!|!=|!==|%|%=|&|&&|&=|\\*|\\*=|\\+|\\+=|,|-|-=|/=|/|:|;|<<|<<=|<=|<|===|==|=|>>>=|>>=|>=|>>>|>>|>|\\?|\\[|\\{|\\(|\\^|\\^=|\\||\\|=|\\|\\||~",
SHEBANG:(e={})=>{const t=/^#![ ]*\//
;return e.binary&&(e.begin=b(t,/.*\b/,e.binary,/\b.*/)),a({scope:"meta",begin:t,
end:/$/,relevance:0,"on:begin":(e,t)=>{0!==e.index&&t.ignoreMatch()}},e)},
BACKSLASH_ESCAPE:k,APOS_STRING_MODE:v,QUOTE_STRING_MODE:O,PHRASAL_WORDS_MODE:{
begin:/\b(a|an|the|are|I'm|isn't|don't|doesn't|won't|but|just|should|pretty|simply|enough|gonna|going|wtf|so|such|will|you|your|they|like|more)\b/
},COMMENT:S,C_LINE_COMMENT_MODE:R,C_BLOCK_COMMENT_MODE:M,HASH_COMMENT_MODE:I,
NUMBER_MODE:{scope:"number",begin:y,relevance:0},C_NUMBER_MODE:{scope:"number",
begin:N,relevance:0},BINARY_NUMBER_MODE:{scope:"number",begin:x,relevance:0},
REGEXP_MODE:{begin:/(?=\/[^/\n]*\/)/,contains:[{scope:"regexp",begin:/\//,
end:/\/[gimuy]*/,illegal:/\n/,contains:[k,{begin:/\[/,end:/\]/,relevance:0,
contains:[k]}]}]},TITLE_MODE:{scope:"title",begin:_,relevance:0},
UNDERSCORE_TITLE_MODE:{scope:"title",begin:w,relevance:0},METHOD_GUARD:{
begin:"\\.\\s*[a-zA-Z_]\\w*",relevance:0},END_SAME_AS_BEGIN:e=>Object.assign(e,{
"on:begin":(e,t)=>{t.data._beginMatch=e[1]},"on:end":(e,t)=>{
t.data._beginMatch!==e[1]&&t.ignoreMatch()}})});function T(e,t){
"."===e.input[e.index-1]&&t.ignoreMatch()}function D(e,t){
void 0!==e.className&&(e.scope=e.className,delete e.className)}function j(e,t){
t&&e.beginKeywords&&(e.begin="\\b("+e.beginKeywords.split(" ").join("|")+")(?!\\.)(?=\\b|\\s)",
e.__beforeBegin=T,e.keywords=e.keywords||e.beginKeywords,delete e.beginKeywords,
void 0===e.relevance&&(e.relevance=0))}function C(e,t){
Array.isArray(e.illegal)&&(e.illegal=p(...e.illegal))}function B(e,t){
if(e.match){
if(e.begin||e.end)throw Error("begin & end are not supported with match")
;e.begin=e.match,delete e.match}}function L(e,t){
void 0===e.relevance&&(e.relevance=1)}const z=(e,t)=>{if(!e.beforeMatch)return
;if(e.starts)throw Error("beforeMatch cannot be used with starts")
;const n=Object.assign({},e);Object.keys(e).forEach((t=>{delete e[t]
})),e.keywords=n.keywords,e.begin=b(n.beforeMatch,u(n.begin)),e.starts={
relevance:0,contains:[Object.assign(n,{endsParent:!0})]
},e.relevance=0,delete n.beforeMatch
},$=["of","and","for","in","not","or","if","then","parent","list","value"]
;function U(e,t,n="keyword"){const s=Object.create(null)
;return"string"==typeof e?i(n,e.split(" ")):Array.isArray(e)?i(n,e):Object.keys(e).forEach((n=>{
Object.assign(s,U(e[n],t,n))})),s;function i(e,n){
t&&(n=n.map((e=>e.toLowerCase()))),n.forEach((t=>{const n=t.split("|")
;s[n[0]]=[e,H(n[0],n[1])]}))}}function H(e,t){
return t?Number(t):(e=>$.includes(e.toLowerCase()))(e)?0:1}const P={},K=e=>{
console.error(e)},G=(e,...t)=>{console.log("WARN: "+e,...t)},F=(e,t)=>{
P[`${e}/${t}`]||(console.log(`Deprecated as of ${e}. ${t}`),P[`${e}/${t}`]=!0)
},Z=Error();function W(e,t,{key:n}){let s=0;const i=e[n],a={},r={}
;for(let e=1;e<=t.length;e++)r[e+s]=i[e],a[e+s]=!0,s+=f(t[e-1])
;e[n]=r,e[n]._emit=a,e[n]._multi=!0}function X(e){(e=>{
e.scope&&"object"==typeof e.scope&&null!==e.scope&&(e.beginScope=e.scope,
delete e.scope)})(e),"string"==typeof e.beginScope&&(e.beginScope={
_wrap:e.beginScope}),"string"==typeof e.endScope&&(e.endScope={_wrap:e.endScope
}),(e=>{if(Array.isArray(e.begin)){
if(e.skip||e.excludeBegin||e.returnBegin)throw K("skip, excludeBegin, returnBegin not compatible with beginScope: {}"),
Z
;if("object"!=typeof e.beginScope||null===e.beginScope)throw K("beginScope must be object"),
Z;W(e,e.begin,{key:"beginScope"}),e.begin=E(e.begin,{joinWith:""})}})(e),(e=>{
if(Array.isArray(e.end)){
if(e.skip||e.excludeEnd||e.returnEnd)throw K("skip, excludeEnd, returnEnd not compatible with endScope: {}"),
Z
;if("object"!=typeof e.endScope||null===e.endScope)throw K("endScope must be object"),
Z;W(e,e.end,{key:"endScope"}),e.end=E(e.end,{joinWith:""})}})(e)}function q(e){
function t(t,n){
return RegExp(d(t),"m"+(e.case_insensitive?"i":"")+(e.unicodeRegex?"u":"")+(n?"g":""))
}class n{constructor(){
this.matchIndexes={},this.regexes=[],this.matchAt=1,this.position=0}
addRule(e,t){
t.position=this.position++,this.matchIndexes[this.matchAt]=t,this.regexes.push([t,e]),
this.matchAt+=f(e)+1}compile(){0===this.regexes.length&&(this.exec=()=>null)
;const e=this.regexes.map((e=>e[1]));this.matcherRe=t(E(e,{joinWith:"|"
}),!0),this.lastIndex=0}exec(e){this.matcherRe.lastIndex=this.lastIndex
;const t=this.matcherRe.exec(e);if(!t)return null
;const n=t.findIndex(((e,t)=>t>0&&void 0!==e)),s=this.matchIndexes[n]
;return t.splice(0,n),Object.assign(t,s)}}class s{constructor(){
this.rules=[],this.multiRegexes=[],
this.count=0,this.lastIndex=0,this.regexIndex=0}getMatcher(e){
if(this.multiRegexes[e])return this.multiRegexes[e];const t=new n
;return this.rules.slice(e).forEach((([e,n])=>t.addRule(e,n))),
t.compile(),this.multiRegexes[e]=t,t}resumingScanAtSamePosition(){
return 0!==this.regexIndex}considerAll(){this.regexIndex=0}addRule(e,t){
this.rules.push([e,t]),"begin"===t.type&&this.count++}exec(e){
const t=this.getMatcher(this.regexIndex);t.lastIndex=this.lastIndex
;let n=t.exec(e)
;if(this.resumingScanAtSamePosition())if(n&&n.index===this.lastIndex);else{
const t=this.getMatcher(0);t.lastIndex=this.lastIndex+1,n=t.exec(e)}
return n&&(this.regexIndex+=n.position+1,
this.regexIndex===this.count&&this.considerAll()),n}}
if(e.compilerExtensions||(e.compilerExtensions=[]),
e.contains&&e.contains.includes("self"))throw Error("ERR: contains `self` is not supported at the top-level of a language.  See documentation.")
;return e.classNameAliases=a(e.classNameAliases||{}),function n(i,r){const o=i
;if(i.isCompiled)return o
;[D,B,X,z].forEach((e=>e(i,r))),e.compilerExtensions.forEach((e=>e(i,r))),
i.__beforeBegin=null,[j,C,L].forEach((e=>e(i,r))),i.isCompiled=!0;let l=null
;return"object"==typeof i.keywords&&i.keywords.$pattern&&(i.keywords=Object.assign({},i.keywords),
l=i.keywords.$pattern,
delete i.keywords.$pattern),l=l||/\w+/,i.keywords&&(i.keywords=U(i.keywords,e.case_insensitive)),
o.keywordPatternRe=t(l,!0),
r&&(i.begin||(i.begin=/\B|\b/),o.beginRe=t(o.begin),i.end||i.endsWithParent||(i.end=/\B|\b/),
i.end&&(o.endRe=t(o.end)),
o.terminatorEnd=d(o.end)||"",i.endsWithParent&&r.terminatorEnd&&(o.terminatorEnd+=(i.end?"|":"")+r.terminatorEnd)),
i.illegal&&(o.illegalRe=t(i.illegal)),
i.contains||(i.contains=[]),i.contains=[].concat(...i.contains.map((e=>(e=>(e.variants&&!e.cachedVariants&&(e.cachedVariants=e.variants.map((t=>a(e,{
variants:null},t)))),e.cachedVariants?e.cachedVariants:V(e)?a(e,{
starts:e.starts?a(e.starts):null
}):Object.isFrozen(e)?a(e):e))("self"===e?i:e)))),i.contains.forEach((e=>{n(e,o)
})),i.starts&&n(i.starts,r),o.matcher=(e=>{const t=new s
;return e.contains.forEach((e=>t.addRule(e.begin,{rule:e,type:"begin"
}))),e.terminatorEnd&&t.addRule(e.terminatorEnd,{type:"end"
}),e.illegal&&t.addRule(e.illegal,{type:"illegal"}),t})(o),o}(e)}function V(e){
return!!e&&(e.endsWithParent||V(e.starts))}class Q extends Error{
constructor(e,t){super(e),this.name="HTMLInjectionError",this.html=t}}
const J=i,Y=a,ee=Symbol("nomatch");var te=(e=>{
const t=Object.create(null),i=Object.create(null),a=[];let r=!0
;const o="Could not find the language '{}', did you forget to load/include a language module?",l={
disableAutodetect:!0,name:"Plain text",contains:[]};let d={
ignoreUnescapedHTML:!1,throwUnescapedHTML:!1,noHighlightRe:/^(no-?highlight)$/i,
languageDetectRe:/\blang(?:uage)?-([\w-]+)\b/i,classPrefix:"hljs-",
cssSelector:"pre code",languages:null,__emitter:c};function f(e){
return d.noHighlightRe.test(e)}function m(e,t,n){let s="",i=""
;"object"==typeof t?(s=e,
n=t.ignoreIllegals,i=t.language):(F("10.7.0","highlight(lang, code, ...args) has been deprecated."),
F("10.7.0","Please use highlight(code, options) instead.\nhttps://github.com/highlightjs/highlight.js/issues/2277"),
i=e,s=t),void 0===n&&(n=!0);const a={code:s,language:i};O("before:highlight",a)
;const r=a.result?a.result:E(a.language,a.code,n)
;return r.code=a.code,O("after:highlight",r),r}function E(e,n,i,a){
const l=Object.create(null);function c(){if(!v.keywords)return void S.addText(R)
;let e=0;v.keywordPatternRe.lastIndex=0;let t=v.keywordPatternRe.exec(R),n=""
;for(;t;){n+=R.substring(e,t.index)
;const i=y.case_insensitive?t[0].toLowerCase():t[0],a=(s=i,v.keywords[s]);if(a){
const[e,s]=a
;if(S.addText(n),n="",l[i]=(l[i]||0)+1,l[i]<=7&&(M+=s),e.startsWith("_"))n+=t[0];else{
const n=y.classNameAliases[e]||e;S.addKeyword(t[0],n)}}else n+=t[0]
;e=v.keywordPatternRe.lastIndex,t=v.keywordPatternRe.exec(R)}var s
;n+=R.substr(e),S.addText(n)}function u(){null!=v.subLanguage?(()=>{
if(""===R)return;let e=null;if("string"==typeof v.subLanguage){
if(!t[v.subLanguage])return void S.addText(R)
;e=E(v.subLanguage,R,!0,O[v.subLanguage]),O[v.subLanguage]=e._top
}else e=_(R,v.subLanguage.length?v.subLanguage:null)
;v.relevance>0&&(M+=e.relevance),S.addSublanguage(e._emitter,e.language)
})():c(),R=""}function g(e,t){let n=1;for(;void 0!==t[n];){if(!e._emit[n]){n++
;continue}const s=y.classNameAliases[e[n]]||e[n],i=t[n]
;s?S.addKeyword(i,s):(R=i,c(),R=""),n++}}function h(e,t){
return e.scope&&"string"==typeof e.scope&&S.openNode(y.classNameAliases[e.scope]||e.scope),
e.beginScope&&(e.beginScope._wrap?(S.addKeyword(R,y.classNameAliases[e.beginScope._wrap]||e.beginScope._wrap),
R=""):e.beginScope._multi&&(g(e.beginScope,t),R="")),v=Object.create(e,{parent:{
value:v}}),v}function b(e,t,n){let i=((e,t)=>{const n=e&&e.exec(t)
;return n&&0===n.index})(e.endRe,n);if(i){if(e["on:end"]){const n=new s(e)
;e["on:end"](t,n),n.isMatchIgnored&&(i=!1)}if(i){
for(;e.endsParent&&e.parent;)e=e.parent;return e}}
if(e.endsWithParent)return b(e.parent,t,n)}function p(e){
return 0===v.matcher.regexIndex?(R+=e[0],1):(T=!0,0)}function f(e){
const t=e[0],s=n.substr(e.index),i=b(v,e,s);if(!i)return ee;const a=v
;v.endScope&&v.endScope._wrap?(u(),
S.addKeyword(t,v.endScope._wrap)):v.endScope&&v.endScope._multi?(u(),
g(v.endScope,e)):a.skip?R+=t:(a.returnEnd||a.excludeEnd||(R+=t),
u(),a.excludeEnd&&(R=t));do{
v.scope&&S.closeNode(),v.skip||v.subLanguage||(M+=v.relevance),v=v.parent
}while(v!==i.parent);return i.starts&&h(i.starts,e),a.returnEnd?0:t.length}
let m={};function w(t,a){const o=a&&a[0];if(R+=t,null==o)return u(),0
;if("begin"===m.type&&"end"===a.type&&m.index===a.index&&""===o){
if(R+=n.slice(a.index,a.index+1),!r){const t=Error(`0 width match regex (${e})`)
;throw t.languageName=e,t.badRule=m.rule,t}return 1}
if(m=a,"begin"===a.type)return(e=>{
const t=e[0],n=e.rule,i=new s(n),a=[n.__beforeBegin,n["on:begin"]]
;for(const n of a)if(n&&(n(e,i),i.isMatchIgnored))return p(t)
;return n.skip?R+=t:(n.excludeBegin&&(R+=t),
u(),n.returnBegin||n.excludeBegin||(R=t)),h(n,e),n.returnBegin?0:t.length})(a)
;if("illegal"===a.type&&!i){
const e=Error('Illegal lexeme "'+o+'" for mode "'+(v.scope||"<unnamed>")+'"')
;throw e.mode=v,e}if("end"===a.type){const e=f(a);if(e!==ee)return e}
if("illegal"===a.type&&""===o)return 1
;if(A>1e5&&A>3*a.index)throw Error("potential infinite loop, way more iterations than matches")
;return R+=o,o.length}const y=x(e)
;if(!y)throw K(o.replace("{}",e)),Error('Unknown language: "'+e+'"')
;const N=q(y);let k="",v=a||N;const O={},S=new d.__emitter(d);(()=>{const e=[]
;for(let t=v;t!==y;t=t.parent)t.scope&&e.unshift(t.scope)
;e.forEach((e=>S.openNode(e)))})();let R="",M=0,I=0,A=0,T=!1;try{
for(v.matcher.considerAll();;){
A++,T?T=!1:v.matcher.considerAll(),v.matcher.lastIndex=I
;const e=v.matcher.exec(n);if(!e)break;const t=w(n.substring(I,e.index),e)
;I=e.index+t}return w(n.substr(I)),S.closeAllNodes(),S.finalize(),k=S.toHTML(),{
language:e,value:k,relevance:M,illegal:!1,_emitter:S,_top:v}}catch(t){
if(t.message&&t.message.includes("Illegal"))return{language:e,value:J(n),
illegal:!0,relevance:0,_illegalBy:{message:t.message,index:I,
context:n.slice(I-100,I+100),mode:t.mode,resultSoFar:k},_emitter:S};if(r)return{
language:e,value:J(n),illegal:!1,relevance:0,errorRaised:t,_emitter:S,_top:v}
;throw t}}function _(e,n){n=n||d.languages||Object.keys(t);const s=(e=>{
const t={value:J(e),illegal:!1,relevance:0,_top:l,_emitter:new d.__emitter(d)}
;return t._emitter.addText(e),t})(e),i=n.filter(x).filter(v).map((t=>E(t,e,!1)))
;i.unshift(s);const a=i.sort(((e,t)=>{
if(e.relevance!==t.relevance)return t.relevance-e.relevance
;if(e.language&&t.language){if(x(e.language).supersetOf===t.language)return 1
;if(x(t.language).supersetOf===e.language)return-1}return 0})),[r,o]=a,c=r
;return c.secondBest=o,c}function w(e){let t=null;const n=(e=>{
let t=e.className+" ";t+=e.parentNode?e.parentNode.className:""
;const n=d.languageDetectRe.exec(t);if(n){const t=x(n[1])
;return t||(G(o.replace("{}",n[1])),
G("Falling back to no-highlight mode for this block.",e)),t?n[1]:"no-highlight"}
return t.split(/\s+/).find((e=>f(e)||x(e)))})(e);if(f(n))return
;if(O("before:highlightElement",{el:e,language:n
}),e.children.length>0&&(d.ignoreUnescapedHTML||(console.warn("One of your code blocks includes unescaped HTML. This is a potentially serious security risk."),
console.warn("https://github.com/highlightjs/highlight.js/issues/2886"),
console.warn(e)),
d.throwUnescapedHTML))throw new Q("One of your code blocks includes unescaped HTML.",e.innerHTML)
;t=e;const s=t.textContent,a=n?m(s,{language:n,ignoreIllegals:!0}):_(s)
;e.innerHTML=a.value,((e,t,n)=>{const s=t&&i[t]||n
;e.classList.add("hljs"),e.classList.add("language-"+s)
})(e,n,a.language),e.result={language:a.language,re:a.relevance,
relevance:a.relevance},a.secondBest&&(e.secondBest={
language:a.secondBest.language,relevance:a.secondBest.relevance
}),O("after:highlightElement",{el:e,result:a,text:s})}let y=!1;function N(){
"loading"!==document.readyState?document.querySelectorAll(d.cssSelector).forEach(w):y=!0
}function x(e){return e=(e||"").toLowerCase(),t[e]||t[i[e]]}
function k(e,{languageName:t}){"string"==typeof e&&(e=[e]),e.forEach((e=>{
i[e.toLowerCase()]=t}))}function v(e){const t=x(e)
;return t&&!t.disableAutodetect}function O(e,t){const n=e;a.forEach((e=>{
e[n]&&e[n](t)}))}
"undefined"!=typeof window&&window.addEventListener&&window.addEventListener("DOMContentLoaded",(()=>{
y&&N()}),!1),Object.assign(e,{highlight:m,highlightAuto:_,highlightAll:N,
highlightElement:w,
highlightBlock:e=>(F("10.7.0","highlightBlock will be removed entirely in v12.0"),
F("10.7.0","Please use highlightElement now."),w(e)),configure:e=>{d=Y(d,e)},
initHighlighting:()=>{
N(),F("10.6.0","initHighlighting() deprecated.  Use highlightAll() now.")},
initHighlightingOnLoad:()=>{
N(),F("10.6.0","initHighlightingOnLoad() deprecated.  Use highlightAll() now.")
},registerLanguage:(n,s)=>{let i=null;try{i=s(e)}catch(e){
if(K("Language definition for '{}' could not be registered.".replace("{}",n)),
!r)throw e;K(e),i=l}
i.name||(i.name=n),t[n]=i,i.rawDefinition=s.bind(null,e),i.aliases&&k(i.aliases,{
languageName:n})},unregisterLanguage:e=>{delete t[e]
;for(const t of Object.keys(i))i[t]===e&&delete i[t]},
listLanguages:()=>Object.keys(t),getLanguage:x,registerAliases:k,
autoDetection:v,inherit:Y,addPlugin:e=>{(e=>{
e["before:highlightBlock"]&&!e["before:highlightElement"]&&(e["before:highlightElement"]=t=>{
e["before:highlightBlock"](Object.assign({block:t.el},t))
}),e["after:highlightBlock"]&&!e["after:highlightElement"]&&(e["after:highlightElement"]=t=>{
e["after:highlightBlock"](Object.assign({block:t.el},t))})})(e),a.push(e)}
}),e.debugMode=()=>{r=!1},e.safeMode=()=>{r=!0
},e.versionString="11.3.1",e.regex={concat:b,lookahead:u,either:p,optional:h,
anyNumberOfTimes:g};for(const e in A)"object"==typeof A[e]&&n(A[e])
;return Object.assign(e,A),e})({}),ne=Object.freeze({__proto__:null,
grmr_sway:e=>{const t={className:"title.function.invoke",relevance:0,
begin:b(/\b/,/(?!let\b)/,e.IDENT_RE,u(/\s*\(/))},n="([u](8|16|32|64))?";return{
name:"Sway",aliases:["sw"],keywords:{$pattern:e.IDENT_RE+"!?",
keyword:["abi","as","asm","break","const","continue","contract","deref","else","enum","fn","for","if","impl","let","library","mod","match","mut","predicate","pub","ref","return","script","Self","self","storage","str","struct","type","trait","use","where","while"],
literal:["true","false","Some","None","Ok","Err"],
built_in:["bool","char","u8","u16","u32","u64","b256","str","Self"]},
illegal:"</",contains:[e.C_LINE_COMMENT_MODE,e.COMMENT("/\\*","\\*/",{
contains:["self"]}),e.inherit(e.QUOTE_STRING_MODE,{begin:/b?"/,illegal:null}),{
className:"string",variants:[{begin:/b?r(#*)"(.|\n)*?"\1(?!#)/},{
begin:/b?'\\?(x\w{2}|u\w{4}|U\w{8}|.)'/}]},{scope:"meta",match:/#\[.*\]/},{
className:"symbol",begin:/'[a-zA-Z_][a-zA-Z0-9_]*/},{className:"number",
variants:[{begin:"\\b0b([01_]+)"+n},{begin:"\\b0o([0-7_]+)"+n},{
begin:"\\b0x([A-Fa-f0-9_]+)"+n},{
begin:"\\b(\\d[\\d_]*(\\.[0-9_]+)?([eE][+-]?[0-9_]+)?)"+n}],relevance:0},{
begin:[/fn/,/\s+/,e.UNDERSCORE_IDENT_RE],className:{1:"keyword",
3:"title.function"}},{
begin:[/(let|const)/,/\s+/,/(?:mut\s+)?/,e.UNDERSCORE_IDENT_RE],className:{
1:"keyword",3:"keyword",4:"variable"}},{
begin:[/type/,/\s+/,e.UNDERSCORE_IDENT_RE],className:{1:"keyword",
3:"title.class"}},{
begin:[/(?:trait|enum|struct|impl|for|library|abi)/,/\s+/,e.UNDERSCORE_IDENT_RE],
className:{1:"keyword",3:"title.class"}},{begin:e.IDENT_RE+"::",keywords:{
keyword:"Self",built_in:[]}},{className:"punctuation",begin:"->"},t]}},
grmr_rust:e=>{const t=e.regex,n={className:"title.function.invoke",relevance:0,
begin:t.concat(/\b/,/(?!let\b)/,e.IDENT_RE,t.lookahead(/\s*\(/))
},s="([ui](8|16|32|64|128|size)|f(32|64))?",i=["drop ","Copy","Send","Sized","Sync","Drop","Fn","FnMut","FnOnce","ToOwned","Clone","Debug","PartialEq","PartialOrd","Eq","Ord","AsRef","AsMut","Into","From","Default","Iterator","Extend","IntoIterator","DoubleEndedIterator","ExactSizeIterator","SliceConcatExt","ToString","assert!","assert_eq!","bitflags!","bytes!","cfg!","col!","concat!","concat_idents!","debug_assert!","debug_assert_eq!","env!","panic!","file!","format!","format_args!","include_bin!","include_str!","line!","local_data_key!","module_path!","option_env!","print!","println!","select!","stringify!","try!","unimplemented!","unreachable!","vec!","write!","writeln!","macro_rules!","assert_ne!","debug_assert_ne!"]
;return{name:"Rust",aliases:["rs"],keywords:{$pattern:e.IDENT_RE+"!?",
type:["i8","i16","i32","i64","i128","isize","u8","u16","u32","u64","u128","usize","f32","f64","str","char","bool","Box","Option","Result","String","Vec"],
keyword:["abstract","as","async","await","become","box","break","const","continue","crate","do","dyn","else","enum","extern","false","final","fn","for","if","impl","in","let","loop","macro","match","mod","move","mut","override","priv","pub","ref","return","self","Self","static","struct","super","trait","true","try","type","typeof","unsafe","unsized","use","virtual","where","while","yield"],
literal:["true","false","Some","None","Ok","Err"],built_in:i},illegal:"</",
contains:[e.C_LINE_COMMENT_MODE,e.COMMENT("/\\*","\\*/",{contains:["self"]
}),e.inherit(e.QUOTE_STRING_MODE,{begin:/b?"/,illegal:null}),{
className:"string",variants:[{begin:/b?r(#*)"(.|\n)*?"\1(?!#)/},{
begin:/b?'\\?(x\w{2}|u\w{4}|U\w{8}|.)'/}]},{className:"symbol",
begin:/'[a-zA-Z_][a-zA-Z0-9_]*/},{className:"number",variants:[{
begin:"\\b0b([01_]+)"+s},{begin:"\\b0o([0-7_]+)"+s},{
begin:"\\b0x([A-Fa-f0-9_]+)"+s},{
begin:"\\b(\\d[\\d_]*(\\.[0-9_]+)?([eE][+-]?[0-9_]+)?)"+s}],relevance:0},{
begin:[/fn/,/\s+/,e.UNDERSCORE_IDENT_RE],className:{1:"keyword",
3:"title.function"}},{className:"meta",begin:"#!?\\[",end:"\\]",contains:[{
className:"string",begin:/"/,end:/"/}]},{
begin:[/let/,/\s+/,/(?:mut\s+)?/,e.UNDERSCORE_IDENT_RE],className:{1:"keyword",
3:"keyword",4:"variable"}},{
begin:[/for/,/\s+/,e.UNDERSCORE_IDENT_RE,/\s+/,/in/],className:{1:"keyword",
3:"variable",5:"keyword"}},{begin:[/type/,/\s+/,e.UNDERSCORE_IDENT_RE],
className:{1:"keyword",3:"title.class"}},{
begin:[/(?:trait|enum|struct|union|impl|for)/,/\s+/,e.UNDERSCORE_IDENT_RE],
className:{1:"keyword",3:"title.class"}},{begin:e.IDENT_RE+"::",keywords:{
keyword:"Self",built_in:i}},{className:"punctuation",begin:"->"},n]}},
grmr_ini:e=>{const t=e.regex,n={className:"number",relevance:0,variants:[{
begin:/([+-]+)?[\d]+_[\d_]+/},{begin:e.NUMBER_RE}]},s=e.COMMENT();s.variants=[{
begin:/;/,end:/$/},{begin:/#/,end:/$/}];const i={className:"variable",
variants:[{begin:/\$[\w\d"][\w\d_]*/},{begin:/\$\{(.*?)\}/}]},a={
className:"literal",begin:/\bon|off|true|false|yes|no\b/},r={className:"string",
contains:[e.BACKSLASH_ESCAPE],variants:[{begin:"'''",end:"'''",relevance:10},{
begin:'"""',end:'"""',relevance:10},{begin:'"',end:'"'},{begin:"'",end:"'"}]
},o={begin:/\[/,end:/\]/,contains:[s,a,i,r,n,"self"],relevance:0
},l=t.either(/[A-Za-z0-9_-]+/,/"(\\"|[^"])*"/,/'[^']*'/);return{
name:"TOML, also INI",aliases:["toml"],case_insensitive:!0,illegal:/\S/,
contains:[s,{className:"section",begin:/\[+/,end:/\]+/},{
begin:t.concat(l,"(\\s*\\.\\s*",l,")*",t.lookahead(/\s*=\s*[^#\s]/)),
className:"attr",starts:{end:/$/,contains:[s,o,a,i,r,n]}}]}},grmr_bash:e=>{
const t=e.regex,n={},s={begin:/\$\{/,end:/\}/,contains:["self",{begin:/:-/,
contains:[n]}]};Object.assign(n,{className:"variable",variants:[{
begin:t.concat(/\$[\w\d#@][\w\d_]*/,"(?![\\w\\d])(?![$])")},s]});const i={
className:"subst",begin:/\$\(/,end:/\)/,contains:[e.BACKSLASH_ESCAPE]},a={
begin:/<<-?\s*(?=\w+)/,starts:{contains:[e.END_SAME_AS_BEGIN({begin:/(\w+)/,
end:/(\w+)/,className:"string"})]}},r={className:"string",begin:/"/,end:/"/,
contains:[e.BACKSLASH_ESCAPE,n,i]};i.contains.push(r);const o={begin:/\$\(\(/,
end:/\)\)/,contains:[{begin:/\d+#[0-9a-f]+/,className:"number"},e.NUMBER_MODE,n]
},l=e.SHEBANG({binary:"(fish|bash|zsh|sh|csh|ksh|tcsh|dash|scsh)",relevance:10
}),c={className:"function",begin:/\w[\w\d_]*\s*\(\s*\)\s*\{/,returnBegin:!0,
contains:[e.inherit(e.TITLE_MODE,{begin:/\w[\w\d_]*/})],relevance:0};return{
name:"Bash",aliases:["sh"],keywords:{$pattern:/\b[a-z._-]+\b/,
keyword:["if","then","else","elif","fi","for","while","in","do","done","case","esac","function"],
literal:["true","false"],
built_in:["break","cd","continue","eval","exec","exit","export","getopts","hash","pwd","readonly","return","shift","test","times","trap","umask","unset","alias","bind","builtin","caller","command","declare","echo","enable","help","let","local","logout","mapfile","printf","read","readarray","source","type","typeset","ulimit","unalias","set","shopt","autoload","bg","bindkey","bye","cap","chdir","clone","comparguments","compcall","compctl","compdescribe","compfiles","compgroups","compquote","comptags","comptry","compvalues","dirs","disable","disown","echotc","echoti","emulate","fc","fg","float","functions","getcap","getln","history","integer","jobs","kill","limit","log","noglob","popd","print","pushd","pushln","rehash","sched","setcap","setopt","stat","suspend","ttyctl","unfunction","unhash","unlimit","unsetopt","vared","wait","whence","where","which","zcompile","zformat","zftp","zle","zmodload","zparseopts","zprof","zpty","zregexparse","zsocket","zstyle","ztcp","chcon","chgrp","chown","chmod","cp","dd","df","dir","dircolors","ln","ls","mkdir","mkfifo","mknod","mktemp","mv","realpath","rm","rmdir","shred","sync","touch","truncate","vdir","b2sum","base32","base64","cat","cksum","comm","csplit","cut","expand","fmt","fold","head","join","md5sum","nl","numfmt","od","paste","ptx","pr","sha1sum","sha224sum","sha256sum","sha384sum","sha512sum","shuf","sort","split","sum","tac","tail","tr","tsort","unexpand","uniq","wc","arch","basename","chroot","date","dirname","du","echo","env","expr","factor","groups","hostid","id","link","logname","nice","nohup","nproc","pathchk","pinky","printenv","printf","pwd","readlink","runcon","seq","sleep","stat","stdbuf","stty","tee","test","timeout","tty","uname","unlink","uptime","users","who","whoami","yes"]
},contains:[l,e.SHEBANG(),c,o,e.HASH_COMMENT_MODE,a,{match:/(\/[a-z._-]+)+/},r,{
className:"",begin:/\\"/},{className:"string",begin:/'/,end:/'/},n]}},
grmr_shell:e=>({name:"Shell Session",aliases:["console","shellsession"],
contains:[{className:"meta",begin:/^\s{0,3}[/~\w\d[\]()@-]*[>%$#][ ]?/,starts:{
end:/[^\\](?=\s*$)/,subLanguage:"bash"}}]}),grmr_json:e=>({name:"JSON",
contains:[{className:"attr",begin:/"(\\.|[^\\"\r\n])*"(?=\s*:)/,relevance:1.01
},{match:/[{}[\],:]/,className:"punctuation",relevance:0},e.QUOTE_STRING_MODE,{
beginKeywords:"true false null"
},e.C_NUMBER_MODE,e.C_LINE_COMMENT_MODE,e.C_BLOCK_COMMENT_MODE],illegal:"\\S"})
});const se=te;for(const e of Object.keys(ne)){
const t=e.replace("grmr_","").replace("_","-");se.registerLanguage(t,ne[e])}
return se}()
;"object"==typeof exports&&"undefined"!=typeof module&&(module.exports=hljs);