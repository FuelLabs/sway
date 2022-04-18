/*!
  Highlight.js v11.3.1 (git: 2e344f51c3)
  (c) 2006-2022 Ivan Sagalaev and other contributors
  License: BSD-3-Clause
 */
var hljs=function(){"use strict";var e={exports:{}};function t(e){
return e instanceof Map?e.clear=e.delete=e.set=()=>{
throw Error("map is read-only")}:e instanceof Set&&(e.add=e.clear=e.delete=()=>{
throw Error("set is read-only")
}),Object.freeze(e),Object.getOwnPropertyNames(e).forEach((n=>{var i=e[n]
;"object"!=typeof i||Object.isFrozen(i)||t(i)})),e}
e.exports=t,e.exports.default=t;var n=e.exports;class i{constructor(e){
void 0===e.data&&(e.data={}),this.data=e.data,this.isMatchIgnored=!1}
ignoreMatch(){this.isMatchIgnored=!0}}function s(e){
return e.replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;").replace(/'/g,"&#x27;")
}function r(e,...t){const n=Object.create(null);for(const t in e)n[t]=e[t]
;return t.forEach((e=>{for(const t in e)n[t]=e[t]})),n}const a=e=>!!e.kind
;class o{constructor(e,t){
this.buffer="",this.classPrefix=t.classPrefix,e.walk(this)}addText(e){
this.buffer+=s(e)}openNode(e){if(!a(e))return;let t=e.kind
;t=e.sublanguage?"language-"+t:((e,{prefix:t})=>{if(e.includes(".")){
const n=e.split(".")
;return[`${t}${n.shift()}`,...n.map(((e,t)=>`${e}${"_".repeat(t+1)}`))].join(" ")
}return`${t}${e}`})(t,{prefix:this.classPrefix}),this.span(t)}closeNode(e){
a(e)&&(this.buffer+="</span>")}value(){return this.buffer}span(e){
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
return new o(this,this.options).value()}finalize(){return!0}}function g(e){
return e?"string"==typeof e?e:e.source:null}function u(e){return b("(?=",e,")")}
function d(e){return b("(?:",e,")*")}function h(e){return b("(?:",e,")?")}
function b(...e){return e.map((e=>g(e))).join("")}function f(...e){const t=(e=>{
const t=e[e.length-1]
;return"object"==typeof t&&t.constructor===Object?(e.splice(e.length-1,1),t):{}
})(e);return"("+(t.capture?"":"?:")+e.map((e=>g(e))).join("|")+")"}
function p(e){return RegExp(e.toString()+"|").exec("").length-1}
const m=/\[(?:[^\\\]]|\\.)*\]|\(\??|\\([1-9][0-9]*)|\\./
;function E(e,{joinWith:t}){let n=0;return e.map((e=>{n+=1;const t=n
;let i=g(e),s="";for(;i.length>0;){const e=m.exec(i);if(!e){s+=i;break}
s+=i.substring(0,e.index),
i=i.substring(e.index+e[0].length),"\\"===e[0][0]&&e[1]?s+="\\"+(Number(e[1])+t):(s+=e[0],
"("===e[0]&&n++)}return s})).map((e=>`(${e})`)).join(t)}
const _="[a-zA-Z]\\w*",y="[a-zA-Z_]\\w*",w="\\b\\d+(\\.\\d+)?",N="(-?)(\\b0[xX][a-fA-F0-9]+|(\\b\\d+(\\.\\d*)?|\\.\\d+)([eE][-+]?\\d+)?)",x="\\b(0b[01]+)",R={
begin:"\\\\[\\s\\S]",relevance:0},v={scope:"string",begin:"'",end:"'",
illegal:"\\n",contains:[R]},O={scope:"string",begin:'"',end:'"',illegal:"\\n",
contains:[R]},k=(e,t,n={})=>{const i=r({scope:"comment",begin:e,end:t,
contains:[]},n);i.contains.push({scope:"doctag",
begin:"[ ]*(?=(TODO|FIXME|NOTE|BUG|OPTIMIZE|HACK|XXX):)",
end:/(TODO|FIXME|NOTE|BUG|OPTIMIZE|HACK|XXX):/,excludeBegin:!0,relevance:0})
;const s=f("I","a","is","so","us","to","at","if","in","it","on",/[A-Za-z]+['](d|ve|re|ll|t|s|n)/,/[A-Za-z]+[-][a-z]+/,/[A-Za-z][a-z]{2,}/)
;return i.contains.push({begin:b(/[ ]+/,"(",s,/[.]?[:]?([.][ ]|[ ])/,"){3}")}),i
},S=k("//","$"),M=k("/\\*","\\*/"),I=k("#","$");var T=Object.freeze({
__proto__:null,MATCH_NOTHING_RE:/\b\B/,IDENT_RE:_,UNDERSCORE_IDENT_RE:y,
NUMBER_RE:w,C_NUMBER_RE:N,BINARY_NUMBER_RE:x,
RE_STARTERS_RE:"!|!=|!==|%|%=|&|&&|&=|\\*|\\*=|\\+|\\+=|,|-|-=|/=|/|:|;|<<|<<=|<=|<|===|==|=|>>>=|>>=|>=|>>>|>>|>|\\?|\\[|\\{|\\(|\\^|\\^=|\\||\\|=|\\|\\||~",
SHEBANG:(e={})=>{const t=/^#![ ]*\//
;return e.binary&&(e.begin=b(t,/.*\b/,e.binary,/\b.*/)),r({scope:"meta",begin:t,
end:/$/,relevance:0,"on:begin":(e,t)=>{0!==e.index&&t.ignoreMatch()}},e)},
BACKSLASH_ESCAPE:R,APOS_STRING_MODE:v,QUOTE_STRING_MODE:O,PHRASAL_WORDS_MODE:{
begin:/\b(a|an|the|are|I'm|isn't|don't|doesn't|won't|but|just|should|pretty|simply|enough|gonna|going|wtf|so|such|will|you|your|they|like|more)\b/
},COMMENT:k,C_LINE_COMMENT_MODE:S,C_BLOCK_COMMENT_MODE:M,HASH_COMMENT_MODE:I,
NUMBER_MODE:{scope:"number",begin:w,relevance:0},C_NUMBER_MODE:{scope:"number",
begin:N,relevance:0},BINARY_NUMBER_MODE:{scope:"number",begin:x,relevance:0},
REGEXP_MODE:{begin:/(?=\/[^/\n]*\/)/,contains:[{scope:"regexp",begin:/\//,
end:/\/[gimuy]*/,illegal:/\n/,contains:[R,{begin:/\[/,end:/\]/,relevance:0,
contains:[R]}]}]},TITLE_MODE:{scope:"title",begin:_,relevance:0},
UNDERSCORE_TITLE_MODE:{scope:"title",begin:y,relevance:0},METHOD_GUARD:{
begin:"\\.\\s*[a-zA-Z_]\\w*",relevance:0},END_SAME_AS_BEGIN:e=>Object.assign(e,{
"on:begin":(e,t)=>{t.data._beginMatch=e[1]},"on:end":(e,t)=>{
t.data._beginMatch!==e[1]&&t.ignoreMatch()}})});function A(e,t){
"."===e.input[e.index-1]&&t.ignoreMatch()}function j(e,t){
void 0!==e.className&&(e.scope=e.className,delete e.className)}function D(e,t){
t&&e.beginKeywords&&(e.begin="\\b("+e.beginKeywords.split(" ").join("|")+")(?!\\.)(?=\\b|\\s)",
e.__beforeBegin=A,e.keywords=e.keywords||e.beginKeywords,delete e.beginKeywords,
void 0===e.relevance&&(e.relevance=0))}function L(e,t){
Array.isArray(e.illegal)&&(e.illegal=f(...e.illegal))}function C(e,t){
if(e.match){
if(e.begin||e.end)throw Error("begin & end are not supported with match")
;e.begin=e.match,delete e.match}}function B(e,t){
void 0===e.relevance&&(e.relevance=1)}const U=(e,t)=>{if(!e.beforeMatch)return
;if(e.starts)throw Error("beforeMatch cannot be used with starts")
;const n=Object.assign({},e);Object.keys(e).forEach((t=>{delete e[t]
})),e.keywords=n.keywords,e.begin=b(n.beforeMatch,u(n.begin)),e.starts={
relevance:0,contains:[Object.assign(n,{endsParent:!0})]
},e.relevance=0,delete n.beforeMatch
},P=["of","and","for","in","not","or","if","then","parent","list","value"]
;function H(e,t,n="keyword"){const i=Object.create(null)
;return"string"==typeof e?s(n,e.split(" ")):Array.isArray(e)?s(n,e):Object.keys(e).forEach((n=>{
Object.assign(i,H(e[n],t,n))})),i;function s(e,n){
t&&(n=n.map((e=>e.toLowerCase()))),n.forEach((t=>{const n=t.split("|")
;i[n[0]]=[e,$(n[0],n[1])]}))}}function $(e,t){
return t?Number(t):(e=>P.includes(e.toLowerCase()))(e)?0:1}const z={},K=e=>{
console.error(e)},F=(e,...t)=>{console.log("WARN: "+e,...t)},Z=(e,t)=>{
z[`${e}/${t}`]||(console.log(`Deprecated as of ${e}. ${t}`),z[`${e}/${t}`]=!0)
},G=Error();function W(e,t,{key:n}){let i=0;const s=e[n],r={},a={}
;for(let e=1;e<=t.length;e++)a[e+i]=s[e],r[e+i]=!0,i+=p(t[e-1])
;e[n]=a,e[n]._emit=r,e[n]._multi=!0}function X(e){(e=>{
e.scope&&"object"==typeof e.scope&&null!==e.scope&&(e.beginScope=e.scope,
delete e.scope)})(e),"string"==typeof e.beginScope&&(e.beginScope={
_wrap:e.beginScope}),"string"==typeof e.endScope&&(e.endScope={_wrap:e.endScope
}),(e=>{if(Array.isArray(e.begin)){
if(e.skip||e.excludeBegin||e.returnBegin)throw K("skip, excludeBegin, returnBegin not compatible with beginScope: {}"),
G
;if("object"!=typeof e.beginScope||null===e.beginScope)throw K("beginScope must be object"),
G;W(e,e.begin,{key:"beginScope"}),e.begin=E(e.begin,{joinWith:""})}})(e),(e=>{
if(Array.isArray(e.end)){
if(e.skip||e.excludeEnd||e.returnEnd)throw K("skip, excludeEnd, returnEnd not compatible with endScope: {}"),
G
;if("object"!=typeof e.endScope||null===e.endScope)throw K("endScope must be object"),
G;W(e,e.end,{key:"endScope"}),e.end=E(e.end,{joinWith:""})}})(e)}function q(e){
function t(t,n){
return RegExp(g(t),"m"+(e.case_insensitive?"i":"")+(e.unicodeRegex?"u":"")+(n?"g":""))
}class n{constructor(){
this.matchIndexes={},this.regexes=[],this.matchAt=1,this.position=0}
addRule(e,t){
t.position=this.position++,this.matchIndexes[this.matchAt]=t,this.regexes.push([t,e]),
this.matchAt+=p(e)+1}compile(){0===this.regexes.length&&(this.exec=()=>null)
;const e=this.regexes.map((e=>e[1]));this.matcherRe=t(E(e,{joinWith:"|"
}),!0),this.lastIndex=0}exec(e){this.matcherRe.lastIndex=this.lastIndex
;const t=this.matcherRe.exec(e);if(!t)return null
;const n=t.findIndex(((e,t)=>t>0&&void 0!==e)),i=this.matchIndexes[n]
;return t.splice(0,n),Object.assign(t,i)}}class i{constructor(){
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
;return e.classNameAliases=r(e.classNameAliases||{}),function n(s,a){const o=s
;if(s.isCompiled)return o
;[j,C,X,U].forEach((e=>e(s,a))),e.compilerExtensions.forEach((e=>e(s,a))),
s.__beforeBegin=null,[D,L,B].forEach((e=>e(s,a))),s.isCompiled=!0;let l=null
;return"object"==typeof s.keywords&&s.keywords.$pattern&&(s.keywords=Object.assign({},s.keywords),
l=s.keywords.$pattern,
delete s.keywords.$pattern),l=l||/\w+/,s.keywords&&(s.keywords=H(s.keywords,e.case_insensitive)),
o.keywordPatternRe=t(l,!0),
a&&(s.begin||(s.begin=/\B|\b/),o.beginRe=t(o.begin),s.end||s.endsWithParent||(s.end=/\B|\b/),
s.end&&(o.endRe=t(o.end)),
o.terminatorEnd=g(o.end)||"",s.endsWithParent&&a.terminatorEnd&&(o.terminatorEnd+=(s.end?"|":"")+a.terminatorEnd)),
s.illegal&&(o.illegalRe=t(s.illegal)),
s.contains||(s.contains=[]),s.contains=[].concat(...s.contains.map((e=>(e=>(e.variants&&!e.cachedVariants&&(e.cachedVariants=e.variants.map((t=>r(e,{
variants:null},t)))),e.cachedVariants?e.cachedVariants:V(e)?r(e,{
starts:e.starts?r(e.starts):null
}):Object.isFrozen(e)?r(e):e))("self"===e?s:e)))),s.contains.forEach((e=>{n(e,o)
})),s.starts&&n(s.starts,a),o.matcher=(e=>{const t=new i
;return e.contains.forEach((e=>t.addRule(e.begin,{rule:e,type:"begin"
}))),e.terminatorEnd&&t.addRule(e.terminatorEnd,{type:"end"
}),e.illegal&&t.addRule(e.illegal,{type:"illegal"}),t})(o),o}(e)}function V(e){
return!!e&&(e.endsWithParent||V(e.starts))}class Q extends Error{
constructor(e,t){super(e),this.name="HTMLInjectionError",this.html=t}}
const J=s,Y=r,ee=Symbol("nomatch");var te=(e=>{
const t=Object.create(null),s=Object.create(null),r=[];let a=!0
;const o="Could not find the language '{}', did you forget to load/include a language module?",l={
disableAutodetect:!0,name:"Plain text",contains:[]};let g={
ignoreUnescapedHTML:!1,throwUnescapedHTML:!1,noHighlightRe:/^(no-?highlight)$/i,
languageDetectRe:/\blang(?:uage)?-([\w-]+)\b/i,classPrefix:"hljs-",
cssSelector:"pre code",languages:null,__emitter:c};function p(e){
return g.noHighlightRe.test(e)}function m(e,t,n){let i="",s=""
;"object"==typeof t?(i=e,
n=t.ignoreIllegals,s=t.language):(Z("10.7.0","highlight(lang, code, ...args) has been deprecated."),
Z("10.7.0","Please use highlight(code, options) instead.\nhttps://github.com/highlightjs/highlight.js/issues/2277"),
s=e,i=t),void 0===n&&(n=!0);const r={code:i,language:s};O("before:highlight",r)
;const a=r.result?r.result:E(r.language,r.code,n)
;return a.code=r.code,O("after:highlight",a),a}function E(e,n,s,r){
const l=Object.create(null);function c(){if(!v.keywords)return void k.addText(S)
;let e=0;v.keywordPatternRe.lastIndex=0;let t=v.keywordPatternRe.exec(S),n=""
;for(;t;){n+=S.substring(e,t.index)
;const s=w.case_insensitive?t[0].toLowerCase():t[0],r=(i=s,v.keywords[i]);if(r){
const[e,i]=r
;if(k.addText(n),n="",l[s]=(l[s]||0)+1,l[s]<=7&&(M+=i),e.startsWith("_"))n+=t[0];else{
const n=w.classNameAliases[e]||e;k.addKeyword(t[0],n)}}else n+=t[0]
;e=v.keywordPatternRe.lastIndex,t=v.keywordPatternRe.exec(S)}var i
;n+=S.substr(e),k.addText(n)}function u(){null!=v.subLanguage?(()=>{
if(""===S)return;let e=null;if("string"==typeof v.subLanguage){
if(!t[v.subLanguage])return void k.addText(S)
;e=E(v.subLanguage,S,!0,O[v.subLanguage]),O[v.subLanguage]=e._top
}else e=_(S,v.subLanguage.length?v.subLanguage:null)
;v.relevance>0&&(M+=e.relevance),k.addSublanguage(e._emitter,e.language)
})():c(),S=""}function d(e,t){let n=1;for(;void 0!==t[n];){if(!e._emit[n]){n++
;continue}const i=w.classNameAliases[e[n]]||e[n],s=t[n]
;i?k.addKeyword(s,i):(S=s,c(),S=""),n++}}function h(e,t){
return e.scope&&"string"==typeof e.scope&&k.openNode(w.classNameAliases[e.scope]||e.scope),
e.beginScope&&(e.beginScope._wrap?(k.addKeyword(S,w.classNameAliases[e.beginScope._wrap]||e.beginScope._wrap),
S=""):e.beginScope._multi&&(d(e.beginScope,t),S="")),v=Object.create(e,{parent:{
value:v}}),v}function b(e,t,n){let s=((e,t)=>{const n=e&&e.exec(t)
;return n&&0===n.index})(e.endRe,n);if(s){if(e["on:end"]){const n=new i(e)
;e["on:end"](t,n),n.isMatchIgnored&&(s=!1)}if(s){
for(;e.endsParent&&e.parent;)e=e.parent;return e}}
if(e.endsWithParent)return b(e.parent,t,n)}function f(e){
return 0===v.matcher.regexIndex?(S+=e[0],1):(A=!0,0)}function p(e){
const t=e[0],i=n.substr(e.index),s=b(v,e,i);if(!s)return ee;const r=v
;v.endScope&&v.endScope._wrap?(u(),
k.addKeyword(t,v.endScope._wrap)):v.endScope&&v.endScope._multi?(u(),
d(v.endScope,e)):r.skip?S+=t:(r.returnEnd||r.excludeEnd||(S+=t),
u(),r.excludeEnd&&(S=t));do{
v.scope&&k.closeNode(),v.skip||v.subLanguage||(M+=v.relevance),v=v.parent
}while(v!==s.parent);return s.starts&&h(s.starts,e),r.returnEnd?0:t.length}
let m={};function y(t,r){const o=r&&r[0];if(S+=t,null==o)return u(),0
;if("begin"===m.type&&"end"===r.type&&m.index===r.index&&""===o){
if(S+=n.slice(r.index,r.index+1),!a){const t=Error(`0 width match regex (${e})`)
;throw t.languageName=e,t.badRule=m.rule,t}return 1}
if(m=r,"begin"===r.type)return(e=>{
const t=e[0],n=e.rule,s=new i(n),r=[n.__beforeBegin,n["on:begin"]]
;for(const n of r)if(n&&(n(e,s),s.isMatchIgnored))return f(t)
;return n.skip?S+=t:(n.excludeBegin&&(S+=t),
u(),n.returnBegin||n.excludeBegin||(S=t)),h(n,e),n.returnBegin?0:t.length})(r)
;if("illegal"===r.type&&!s){
const e=Error('Illegal lexeme "'+o+'" for mode "'+(v.scope||"<unnamed>")+'"')
;throw e.mode=v,e}if("end"===r.type){const e=p(r);if(e!==ee)return e}
if("illegal"===r.type&&""===o)return 1
;if(T>1e5&&T>3*r.index)throw Error("potential infinite loop, way more iterations than matches")
;return S+=o,o.length}const w=x(e)
;if(!w)throw K(o.replace("{}",e)),Error('Unknown language: "'+e+'"')
;const N=q(w);let R="",v=r||N;const O={},k=new g.__emitter(g);(()=>{const e=[]
;for(let t=v;t!==w;t=t.parent)t.scope&&e.unshift(t.scope)
;e.forEach((e=>k.openNode(e)))})();let S="",M=0,I=0,T=0,A=!1;try{
for(v.matcher.considerAll();;){
T++,A?A=!1:v.matcher.considerAll(),v.matcher.lastIndex=I
;const e=v.matcher.exec(n);if(!e)break;const t=y(n.substring(I,e.index),e)
;I=e.index+t}return y(n.substr(I)),k.closeAllNodes(),k.finalize(),R=k.toHTML(),{
language:e,value:R,relevance:M,illegal:!1,_emitter:k,_top:v}}catch(t){
if(t.message&&t.message.includes("Illegal"))return{language:e,value:J(n),
illegal:!0,relevance:0,_illegalBy:{message:t.message,index:I,
context:n.slice(I-100,I+100),mode:t.mode,resultSoFar:R},_emitter:k};if(a)return{
language:e,value:J(n),illegal:!1,relevance:0,errorRaised:t,_emitter:k,_top:v}
;throw t}}function _(e,n){n=n||g.languages||Object.keys(t);const i=(e=>{
const t={value:J(e),illegal:!1,relevance:0,_top:l,_emitter:new g.__emitter(g)}
;return t._emitter.addText(e),t})(e),s=n.filter(x).filter(v).map((t=>E(t,e,!1)))
;s.unshift(i);const r=s.sort(((e,t)=>{
if(e.relevance!==t.relevance)return t.relevance-e.relevance
;if(e.language&&t.language){if(x(e.language).supersetOf===t.language)return 1
;if(x(t.language).supersetOf===e.language)return-1}return 0})),[a,o]=r,c=a
;return c.secondBest=o,c}function y(e){let t=null;const n=(e=>{
let t=e.className+" ";t+=e.parentNode?e.parentNode.className:""
;const n=g.languageDetectRe.exec(t);if(n){const t=x(n[1])
;return t||(F(o.replace("{}",n[1])),
F("Falling back to no-highlight mode for this block.",e)),t?n[1]:"no-highlight"}
return t.split(/\s+/).find((e=>p(e)||x(e)))})(e);if(p(n))return
;if(O("before:highlightElement",{el:e,language:n
}),e.children.length>0&&(g.ignoreUnescapedHTML||(console.warn("One of your code blocks includes unescaped HTML. This is a potentially serious security risk."),
console.warn("https://github.com/highlightjs/highlight.js/wiki/security"),
console.warn("The element with unescaped HTML:"),
console.warn(e)),g.throwUnescapedHTML))throw new Q("One of your code blocks includes unescaped HTML.",e.innerHTML)
;t=e;const i=t.textContent,r=n?m(i,{language:n,ignoreIllegals:!0}):_(i)
;e.innerHTML=r.value,((e,t,n)=>{const i=t&&s[t]||n
;e.classList.add("hljs"),e.classList.add("language-"+i)
})(e,n,r.language),e.result={language:r.language,re:r.relevance,
relevance:r.relevance},r.secondBest&&(e.secondBest={
language:r.secondBest.language,relevance:r.secondBest.relevance
}),O("after:highlightElement",{el:e,result:r,text:i})}let w=!1;function N(){
"loading"!==document.readyState?document.querySelectorAll(g.cssSelector).forEach(y):w=!0
}function x(e){return e=(e||"").toLowerCase(),t[e]||t[s[e]]}
function R(e,{languageName:t}){"string"==typeof e&&(e=[e]),e.forEach((e=>{
s[e.toLowerCase()]=t}))}function v(e){const t=x(e)
;return t&&!t.disableAutodetect}function O(e,t){const n=e;r.forEach((e=>{
e[n]&&e[n](t)}))}
"undefined"!=typeof window&&window.addEventListener&&window.addEventListener("DOMContentLoaded",(()=>{
w&&N()}),!1),Object.assign(e,{highlight:m,highlightAuto:_,highlightAll:N,
highlightElement:y,
highlightBlock:e=>(Z("10.7.0","highlightBlock will be removed entirely in v12.0"),
Z("10.7.0","Please use highlightElement now."),y(e)),configure:e=>{g=Y(g,e)},
initHighlighting:()=>{
N(),Z("10.6.0","initHighlighting() deprecated.  Use highlightAll() now.")},
initHighlightingOnLoad:()=>{
N(),Z("10.6.0","initHighlightingOnLoad() deprecated.  Use highlightAll() now.")
},registerLanguage:(n,i)=>{let s=null;try{s=i(e)}catch(e){
if(K("Language definition for '{}' could not be registered.".replace("{}",n)),
!a)throw e;K(e),s=l}
s.name||(s.name=n),t[n]=s,s.rawDefinition=i.bind(null,e),s.aliases&&R(s.aliases,{
languageName:n})},unregisterLanguage:e=>{delete t[e]
;for(const t of Object.keys(s))s[t]===e&&delete s[t]},
listLanguages:()=>Object.keys(t),getLanguage:x,registerAliases:R,
autoDetection:v,inherit:Y,addPlugin:e=>{(e=>{
e["before:highlightBlock"]&&!e["before:highlightElement"]&&(e["before:highlightElement"]=t=>{
e["before:highlightBlock"](Object.assign({block:t.el},t))
}),e["after:highlightBlock"]&&!e["after:highlightElement"]&&(e["after:highlightElement"]=t=>{
e["after:highlightBlock"](Object.assign({block:t.el},t))})})(e),r.push(e)}
}),e.debugMode=()=>{a=!1},e.safeMode=()=>{a=!0
},e.versionString="11.3.1",e.regex={concat:b,lookahead:u,either:f,optional:h,
anyNumberOfTimes:d};for(const e in T)"object"==typeof T[e]&&n(T[e])
;return Object.assign(e,T),e})({}),ne=Object.freeze({__proto__:null,
grmr_sway:e=>{const t={className:"title.function.invoke",relevance:0,
begin:b(/\b/,/(?!let\b)/,e.IDENT_RE,u(/\s*\(/))},n="([u](8|16|32|64))?";return{
name:"Sway",aliases:["sw"],keywords:{$pattern:e.IDENT_RE+"!?",
keyword:["abi","as","asm","const","contract","deref","enum","fn","if","impl","let","library","match","mut","else","predicate","ref","return","script","Self","self","str","struct","trait","use","where","while"],
literal:["true","false"],
built_in:["bool","char","u8","u16","u32","u64","b256","str","Self"]},
illegal:"</",contains:[e.C_LINE_COMMENT_MODE,e.COMMENT("/\\*","\\*/",{
contains:["self"]}),e.inherit(e.QUOTE_STRING_MODE,{begin:/b?"/,illegal:null}),{
className:"string",variants:[{begin:/b?r(#*)"(.|\n)*?"\1(?!#)/},{
begin:/b?'\\?(x\w{2}|u\w{4}|U\w{8}|.)'/}]},{className:"symbol",
begin:/'[a-zA-Z_][a-zA-Z0-9_]*/},{className:"number",variants:[{
begin:"\\b0b([01_]+)"+n},{begin:"\\b0o([0-7_]+)"+n},{
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
},i="([ui](8|16|32|64|128|size)|f(32|64))?",s=["drop ","Copy","Send","Sized","Sync","Drop","Fn","FnMut","FnOnce","ToOwned","Clone","Debug","PartialEq","PartialOrd","Eq","Ord","AsRef","AsMut","Into","From","Default","Iterator","Extend","IntoIterator","DoubleEndedIterator","ExactSizeIterator","SliceConcatExt","ToString","assert!","assert_eq!","bitflags!","bytes!","cfg!","col!","concat!","concat_idents!","debug_assert!","debug_assert_eq!","env!","panic!","file!","format!","format_args!","include_bin!","include_str!","line!","local_data_key!","module_path!","option_env!","print!","println!","select!","stringify!","try!","unimplemented!","unreachable!","vec!","write!","writeln!","macro_rules!","assert_ne!","debug_assert_ne!"]
;return{name:"Rust",aliases:["rs"],keywords:{$pattern:e.IDENT_RE+"!?",
type:["i8","i16","i32","i64","i128","isize","u8","u16","u32","u64","u128","usize","f32","f64","str","char","bool","Box","Option","Result","String","Vec"],
keyword:["abstract","as","async","await","become","box","break","const","continue","crate","do","dyn","else","enum","extern","false","final","fn","for","if","impl","in","let","loop","macro","match","mod","move","mut","override","priv","pub","ref","return","self","Self","static","struct","super","trait","true","try","type","typeof","unsafe","unsized","use","virtual","where","while","yield"],
literal:["true","false","Some","None","Ok","Err"],built_in:s},illegal:"</",
contains:[e.C_LINE_COMMENT_MODE,e.COMMENT("/\\*","\\*/",{contains:["self"]
}),e.inherit(e.QUOTE_STRING_MODE,{begin:/b?"/,illegal:null}),{
className:"string",variants:[{begin:/b?r(#*)"(.|\n)*?"\1(?!#)/},{
begin:/b?'\\?(x\w{2}|u\w{4}|U\w{8}|.)'/}]},{className:"symbol",
begin:/'[a-zA-Z_][a-zA-Z0-9_]*/},{className:"number",variants:[{
begin:"\\b0b([01_]+)"+i},{begin:"\\b0o([0-7_]+)"+i},{
begin:"\\b0x([A-Fa-f0-9_]+)"+i},{
begin:"\\b(\\d[\\d_]*(\\.[0-9_]+)?([eE][+-]?[0-9_]+)?)"+i}],relevance:0},{
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
keyword:"Self",built_in:s}},{className:"punctuation",begin:"->"},n]}},
grmr_ini:e=>{const t=e.regex,n={className:"number",relevance:0,variants:[{
begin:/([+-]+)?[\d]+_[\d_]+/},{begin:e.NUMBER_RE}]},i=e.COMMENT();i.variants=[{
begin:/;/,end:/$/},{begin:/#/,end:/$/}];const s={className:"variable",
variants:[{begin:/\$[\w\d"][\w\d_]*/},{begin:/\$\{(.*?)\}/}]},r={
className:"literal",begin:/\bon|off|true|false|yes|no\b/},a={className:"string",
contains:[e.BACKSLASH_ESCAPE],variants:[{begin:"'''",end:"'''",relevance:10},{
begin:'"""',end:'"""',relevance:10},{begin:'"',end:'"'},{begin:"'",end:"'"}]
},o={begin:/\[/,end:/\]/,contains:[i,r,s,a,n,"self"],relevance:0
},l=t.either(/[A-Za-z0-9_-]+/,/"(\\"|[^"])*"/,/'[^']*'/);return{
name:"TOML, also INI",aliases:["toml"],case_insensitive:!0,illegal:/\S/,
contains:[i,{className:"section",begin:/\[+/,end:/\]+/},{
begin:t.concat(l,"(\\s*\\.\\s*",l,")*",t.lookahead(/\s*=\s*[^#\s]/)),
className:"attr",starts:{end:/$/,contains:[i,o,r,s,a,n]}}]}}});const ie=te
;for(const e of Object.keys(ne)){const t=e.replace("grmr_","").replace("_","-")
;ie.registerLanguage(t,ne[e])}return ie}()
;"object"==typeof exports&&"undefined"!=typeof module&&(module.exports=hljs);