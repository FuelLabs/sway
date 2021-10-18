/*!
  Highlight.js v11.1.0 (git: bd548da78a)
  (c) 2006-2021 Ivan Sagalaev and other contributors
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
    ignoreMatch(){this.isMatchIgnored=!0}}function r(e){
    return e.replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;").replace(/'/g,"&#x27;")
    }function s(e,...t){const n=Object.create(null);for(const t in e)n[t]=e[t]
    ;return t.forEach((e=>{for(const t in e)n[t]=e[t]})),n}const o=e=>!!e.kind
    ;class a{constructor(e,t){
    this.buffer="",this.classPrefix=t.classPrefix,e.walk(this)}addText(e){
    this.buffer+=r(e)}openNode(e){if(!o(e))return;let t=e.kind
    ;t=e.sublanguage?"language-"+t:((e,{prefix:t})=>{if(e.includes(".")){
    const n=e.split(".")
    ;return[`${t}${n.shift()}`,...n.map(((e,t)=>`${e}${"_".repeat(t+1)}`))].join(" ")
    }return`${t}${e}`})(t,{prefix:this.classPrefix}),this.span(t)}closeNode(e){
    o(e)&&(this.buffer+="</span>")}value(){return this.buffer}span(e){
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
    return new a(this,this.options).value()}finalize(){return!0}}function g(e){
    return e?"string"==typeof e?e:e.source:null}function d(e){return u("(?=",e,")")}
    function u(...e){return e.map((e=>g(e))).join("")}function h(...e){
    return"("+((e=>{const t=e[e.length-1]
    ;return"object"==typeof t&&t.constructor===Object?(e.splice(e.length-1,1),t):{}
    })(e).capture?"":"?:")+e.map((e=>g(e))).join("|")+")"}function f(e){
    return RegExp(e.toString()+"|").exec("").length-1}
    const p=/\[(?:[^\\\]]|\\.)*\]|\(\??|\\([1-9][0-9]*)|\\./
    ;function b(e,{joinWith:t}){let n=0;return e.map((e=>{n+=1;const t=n
    ;let i=g(e),r="";for(;i.length>0;){const e=p.exec(i);if(!e){r+=i;break}
    r+=i.substring(0,e.index),
    i=i.substring(e.index+e[0].length),"\\"===e[0][0]&&e[1]?r+="\\"+(Number(e[1])+t):(r+=e[0],
    "("===e[0]&&n++)}return r})).map((e=>`(${e})`)).join(t)}
    const m="[a-zA-Z]\\w*",E="[a-zA-Z_]\\w*",_="\\b\\d+(\\.\\d+)?",y="(-?)(\\b0[xX][a-fA-F0-9]+|(\\b\\d+(\\.\\d*)?|\\.\\d+)([eE][-+]?\\d+)?)",w="\\b(0b[01]+)",x={
    begin:"\\\\[\\s\\S]",relevance:0},N={scope:"string",begin:"'",end:"'",
    illegal:"\\n",contains:[x]},O={scope:"string",begin:'"',end:'"',illegal:"\\n",
    contains:[x]},k=(e,t,n={})=>{const i=s({scope:"comment",begin:e,end:t,
    contains:[]},n);i.contains.push({scope:"doctag",
    begin:"[ ]*(?=(TODO|FIXME|NOTE|BUG|OPTIMIZE|HACK|XXX):)",
    end:/(TODO|FIXME|NOTE|BUG|OPTIMIZE|HACK|XXX):/,excludeBegin:!0,relevance:0})
    ;const r=h("I","a","is","so","us","to","at","if","in","it","on",/[A-Za-z]+['](d|ve|re|ll|t|s|n)/,/[A-Za-z]+[-][a-z]+/,/[A-Za-z][a-z]{2,}/)
    ;return i.contains.push({begin:u(/[ ]+/,"(",r,/[.]?[:]?([.][ ]|[ ])/,"){3}")}),i
    },v=k("//","$"),R=k("/\\*","\\*/"),S=k("#","$");var M=Object.freeze({
    __proto__:null,MATCH_NOTHING_RE:/\b\B/,IDENT_RE:m,UNDERSCORE_IDENT_RE:E,
    NUMBER_RE:_,C_NUMBER_RE:y,BINARY_NUMBER_RE:w,
    RE_STARTERS_RE:"!|!=|!==|%|%=|&|&&|&=|\\*|\\*=|\\+|\\+=|,|-|-=|/=|/|:|;|<<|<<=|<=|<|===|==|=|>>>=|>>=|>=|>>>|>>|>|\\?|\\[|\\{|\\(|\\^|\\^=|\\||\\|=|\\|\\||~",
    SHEBANG:(e={})=>{const t=/^#![ ]*\//
    ;return e.binary&&(e.begin=u(t,/.*\b/,e.binary,/\b.*/)),s({scope:"meta",begin:t,
    end:/$/,relevance:0,"on:begin":(e,t)=>{0!==e.index&&t.ignoreMatch()}},e)},
    BACKSLASH_ESCAPE:x,APOS_STRING_MODE:N,QUOTE_STRING_MODE:O,PHRASAL_WORDS_MODE:{
    begin:/\b(a|an|the|are|I'm|isn't|don't|doesn't|won't|but|just|should|pretty|simply|enough|gonna|going|wtf|so|such|will|you|your|they|like|more)\b/
    },COMMENT:k,C_LINE_COMMENT_MODE:v,C_BLOCK_COMMENT_MODE:R,HASH_COMMENT_MODE:S,
    NUMBER_MODE:{scope:"number",begin:_,relevance:0},C_NUMBER_MODE:{scope:"number",
    begin:y,relevance:0},BINARY_NUMBER_MODE:{scope:"number",begin:w,relevance:0},
    REGEXP_MODE:{begin:/(?=\/[^/\n]*\/)/,contains:[{scope:"regexp",begin:/\//,
    end:/\/[gimuy]*/,illegal:/\n/,contains:[x,{begin:/\[/,end:/\]/,relevance:0,
    contains:[x]}]}]},TITLE_MODE:{scope:"title",begin:m,relevance:0},
    UNDERSCORE_TITLE_MODE:{scope:"title",begin:E,relevance:0},METHOD_GUARD:{
    begin:"\\.\\s*[a-zA-Z_]\\w*",relevance:0},END_SAME_AS_BEGIN:e=>Object.assign(e,{
    "on:begin":(e,t)=>{t.data._beginMatch=e[1]},"on:end":(e,t)=>{
    t.data._beginMatch!==e[1]&&t.ignoreMatch()}})});function A(e,t){
    "."===e.input[e.index-1]&&t.ignoreMatch()}function j(e,t){
    void 0!==e.className&&(e.scope=e.className,delete e.className)}function I(e,t){
    t&&e.beginKeywords&&(e.begin="\\b("+e.beginKeywords.split(" ").join("|")+")(?!\\.)(?=\\b|\\s)",
    e.__beforeBegin=A,e.keywords=e.keywords||e.beginKeywords,delete e.beginKeywords,
    void 0===e.relevance&&(e.relevance=0))}function T(e,t){
    Array.isArray(e.illegal)&&(e.illegal=h(...e.illegal))}function B(e,t){
    if(e.match){
    if(e.begin||e.end)throw Error("begin & end are not supported with match")
    ;e.begin=e.match,delete e.match}}function D(e,t){
    void 0===e.relevance&&(e.relevance=1)}const L=(e,t)=>{if(!e.beforeMatch)return
    ;if(e.starts)throw Error("beforeMatch cannot be used with starts")
    ;const n=Object.assign({},e);Object.keys(e).forEach((t=>{delete e[t]
    })),e.keywords=n.keywords,e.begin=u(n.beforeMatch,d(n.begin)),e.starts={
    relevance:0,contains:[Object.assign(n,{endsParent:!0})]
    },e.relevance=0,delete n.beforeMatch
    },C=["of","and","for","in","not","or","if","then","parent","list","value"]
    ;function P(e,t,n="keyword"){const i=Object.create(null)
    ;return"string"==typeof e?r(n,e.split(" ")):Array.isArray(e)?r(n,e):Object.keys(e).forEach((n=>{
    Object.assign(i,P(e[n],t,n))})),i;function r(e,n){
    t&&(n=n.map((e=>e.toLowerCase()))),n.forEach((t=>{const n=t.split("|")
    ;i[n[0]]=[e,U(n[0],n[1])]}))}}function U(e,t){
    return t?Number(t):(e=>C.includes(e.toLowerCase()))(e)?0:1}const $={},H=e=>{
    console.error(e)},z=(e,...t)=>{console.log("WARN: "+e,...t)},K=(e,t)=>{
    $[`${e}/${t}`]||(console.log(`Deprecated as of ${e}. ${t}`),$[`${e}/${t}`]=!0)
    },W=Error();function G(e,t,{key:n}){let i=0;const r=e[n],s={},o={}
    ;for(let e=1;e<=t.length;e++)o[e+i]=r[e],s[e+i]=!0,i+=f(t[e-1])
    ;e[n]=o,e[n]._emit=s,e[n]._multi=!0}function X(e){(e=>{
    e.scope&&"object"==typeof e.scope&&null!==e.scope&&(e.beginScope=e.scope,
    delete e.scope)})(e),"string"==typeof e.beginScope&&(e.beginScope={
    _wrap:e.beginScope}),"string"==typeof e.endScope&&(e.endScope={_wrap:e.endScope
    }),(e=>{if(Array.isArray(e.begin)){
    if(e.skip||e.excludeBegin||e.returnBegin)throw H("skip, excludeBegin, returnBegin not compatible with beginScope: {}"),
    W
    ;if("object"!=typeof e.beginScope||null===e.beginScope)throw H("beginScope must be object"),
    W;G(e,e.begin,{key:"beginScope"}),e.begin=b(e.begin,{joinWith:""})}})(e),(e=>{
    if(Array.isArray(e.end)){
    if(e.skip||e.excludeEnd||e.returnEnd)throw H("skip, excludeEnd, returnEnd not compatible with endScope: {}"),
    W
    ;if("object"!=typeof e.endScope||null===e.endScope)throw H("endScope must be object"),
    W;G(e,e.end,{key:"endScope"}),e.end=b(e.end,{joinWith:""})}})(e)}function Z(e){
    function t(t,n){return RegExp(g(t),"m"+(e.case_insensitive?"i":"")+(n?"g":""))}
    class n{constructor(){
    this.matchIndexes={},this.regexes=[],this.matchAt=1,this.position=0}
    addRule(e,t){
    t.position=this.position++,this.matchIndexes[this.matchAt]=t,this.regexes.push([t,e]),
    this.matchAt+=f(e)+1}compile(){0===this.regexes.length&&(this.exec=()=>null)
    ;const e=this.regexes.map((e=>e[1]));this.matcherRe=t(b(e,{joinWith:"|"
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
    ;return e.classNameAliases=s(e.classNameAliases||{}),function n(r,o){const a=r
    ;if(r.isCompiled)return a
    ;[j,B,X,L].forEach((e=>e(r,o))),e.compilerExtensions.forEach((e=>e(r,o))),
    r.__beforeBegin=null,[I,T,D].forEach((e=>e(r,o))),r.isCompiled=!0;let l=null
    ;return"object"==typeof r.keywords&&r.keywords.$pattern&&(r.keywords=Object.assign({},r.keywords),
    l=r.keywords.$pattern,
    delete r.keywords.$pattern),l=l||/\w+/,r.keywords&&(r.keywords=P(r.keywords,e.case_insensitive)),
    a.keywordPatternRe=t(l,!0),
    o&&(r.begin||(r.begin=/\B|\b/),a.beginRe=t(r.begin),r.end||r.endsWithParent||(r.end=/\B|\b/),
    r.end&&(a.endRe=t(r.end)),
    a.terminatorEnd=g(r.end)||"",r.endsWithParent&&o.terminatorEnd&&(a.terminatorEnd+=(r.end?"|":"")+o.terminatorEnd)),
    r.illegal&&(a.illegalRe=t(r.illegal)),
    r.contains||(r.contains=[]),r.contains=[].concat(...r.contains.map((e=>(e=>(e.variants&&!e.cachedVariants&&(e.cachedVariants=e.variants.map((t=>s(e,{
    variants:null},t)))),e.cachedVariants?e.cachedVariants:F(e)?s(e,{
    starts:e.starts?s(e.starts):null
    }):Object.isFrozen(e)?s(e):e))("self"===e?r:e)))),r.contains.forEach((e=>{n(e,a)
    })),r.starts&&n(r.starts,o),a.matcher=(e=>{const t=new i
    ;return e.contains.forEach((e=>t.addRule(e.begin,{rule:e,type:"begin"
    }))),e.terminatorEnd&&t.addRule(e.terminatorEnd,{type:"end"
    }),e.illegal&&t.addRule(e.illegal,{type:"illegal"}),t})(a),a}(e)}function F(e){
    return!!e&&(e.endsWithParent||F(e.starts))}const V=r,q=s,J=Symbol("nomatch")
    ;var Q=(e=>{const t=Object.create(null),r=Object.create(null),s=[];let o=!0
    ;const a="Could not find the language '{}', did you forget to load/include a language module?",l={
    disableAutodetect:!0,name:"Plain text",contains:[]};let g={
    ignoreUnescapedHTML:!1,noHighlightRe:/^(no-?highlight)$/i,
    languageDetectRe:/\blang(?:uage)?-([\w-]+)\b/i,classPrefix:"hljs-",
    cssSelector:"pre code",languages:null,__emitter:c};function d(e){
    return g.noHighlightRe.test(e)}function u(e,t,n){let i="",r=""
    ;"object"==typeof t?(i=e,
    n=t.ignoreIllegals,r=t.language):(K("10.7.0","highlight(lang, code, ...args) has been deprecated."),
    K("10.7.0","Please use highlight(code, options) instead.\nhttps://github.com/highlightjs/highlight.js/issues/2277"),
    r=e,i=t),void 0===n&&(n=!0);const s={code:i,language:r};w("before:highlight",s)
    ;const o=s.result?s.result:h(s.language,s.code,n)
    ;return o.code=s.code,w("after:highlight",o),o}function h(e,n,r,s){
    const l=Object.create(null);function c(){if(!k.keywords)return void R.addText(S)
    ;let e=0;k.keywordPatternRe.lastIndex=0;let t=k.keywordPatternRe.exec(S),n=""
    ;for(;t;){n+=S.substring(e,t.index)
    ;const r=x.case_insensitive?t[0].toLowerCase():t[0],s=(i=r,k.keywords[i]);if(s){
    const[e,i]=s
    ;if(R.addText(n),n="",l[r]=(l[r]||0)+1,l[r]<=7&&(M+=i),e.startsWith("_"))n+=t[0];else{
    const n=x.classNameAliases[e]||e;R.addKeyword(t[0],n)}}else n+=t[0]
    ;e=k.keywordPatternRe.lastIndex,t=k.keywordPatternRe.exec(S)}var i
    ;n+=S.substr(e),R.addText(n)}function d(){null!=k.subLanguage?(()=>{
    if(""===S)return;let e=null;if("string"==typeof k.subLanguage){
    if(!t[k.subLanguage])return void R.addText(S)
    ;e=h(k.subLanguage,S,!0,v[k.subLanguage]),v[k.subLanguage]=e._top
    }else e=f(S,k.subLanguage.length?k.subLanguage:null)
    ;k.relevance>0&&(M+=e.relevance),R.addSublanguage(e._emitter,e.language)
    })():c(),S=""}function u(e,t){let n=1;for(;void 0!==t[n];){if(!e._emit[n]){n++
    ;continue}const i=x.classNameAliases[e[n]]||e[n],r=t[n]
    ;i?R.addKeyword(r,i):(S=r,c(),S=""),n++}}function p(e,t){
    return e.scope&&"string"==typeof e.scope&&R.openNode(x.classNameAliases[e.scope]||e.scope),
    e.beginScope&&(e.beginScope._wrap?(R.addKeyword(S,x.classNameAliases[e.beginScope._wrap]||e.beginScope._wrap),
    S=""):e.beginScope._multi&&(u(e.beginScope,t),S="")),k=Object.create(e,{parent:{
    value:k}}),k}function b(e,t,n){let r=((e,t)=>{const n=e&&e.exec(t)
    ;return n&&0===n.index})(e.endRe,n);if(r){if(e["on:end"]){const n=new i(e)
    ;e["on:end"](t,n),n.isMatchIgnored&&(r=!1)}if(r){
    for(;e.endsParent&&e.parent;)e=e.parent;return e}}
    if(e.endsWithParent)return b(e.parent,t,n)}function m(e){
    return 0===k.matcher.regexIndex?(S+=e[0],1):(I=!0,0)}function _(e){
    const t=e[0],i=n.substr(e.index),r=b(k,e,i);if(!r)return J;const s=k
    ;k.endScope&&k.endScope._wrap?(d(),
    R.addKeyword(t,k.endScope._wrap)):k.endScope&&k.endScope._multi?(d(),
    u(k.endScope,e)):s.skip?S+=t:(s.returnEnd||s.excludeEnd||(S+=t),
    d(),s.excludeEnd&&(S=t));do{
    k.scope&&R.closeNode(),k.skip||k.subLanguage||(M+=k.relevance),k=k.parent
    }while(k!==r.parent);return r.starts&&p(r.starts,e),s.returnEnd?0:t.length}
    let y={};function w(t,s){const a=s&&s[0];if(S+=t,null==a)return d(),0
    ;if("begin"===y.type&&"end"===s.type&&y.index===s.index&&""===a){
    if(S+=n.slice(s.index,s.index+1),!o){const t=Error(`0 width match regex (${e})`)
    ;throw t.languageName=e,t.badRule=y.rule,t}return 1}
    if(y=s,"begin"===s.type)return(e=>{
    const t=e[0],n=e.rule,r=new i(n),s=[n.__beforeBegin,n["on:begin"]]
    ;for(const n of s)if(n&&(n(e,r),r.isMatchIgnored))return m(t)
    ;return n.skip?S+=t:(n.excludeBegin&&(S+=t),
    d(),n.returnBegin||n.excludeBegin||(S=t)),p(n,e),n.returnBegin?0:t.length})(s)
    ;if("illegal"===s.type&&!r){
    const e=Error('Illegal lexeme "'+a+'" for mode "'+(k.scope||"<unnamed>")+'"')
    ;throw e.mode=k,e}if("end"===s.type){const e=_(s);if(e!==J)return e}
    if("illegal"===s.type&&""===a)return 1
    ;if(j>1e5&&j>3*s.index)throw Error("potential infinite loop, way more iterations than matches")
    ;return S+=a,a.length}const x=E(e)
    ;if(!x)throw H(a.replace("{}",e)),Error('Unknown language: "'+e+'"')
    ;const N=Z(x);let O="",k=s||N;const v={},R=new g.__emitter(g);(()=>{const e=[]
    ;for(let t=k;t!==x;t=t.parent)t.scope&&e.unshift(t.scope)
    ;e.forEach((e=>R.openNode(e)))})();let S="",M=0,A=0,j=0,I=!1;try{
    for(k.matcher.considerAll();;){
    j++,I?I=!1:k.matcher.considerAll(),k.matcher.lastIndex=A
    ;const e=k.matcher.exec(n);if(!e)break;const t=w(n.substring(A,e.index),e)
    ;A=e.index+t}return w(n.substr(A)),R.closeAllNodes(),R.finalize(),O=R.toHTML(),{
    language:e,value:O,relevance:M,illegal:!1,_emitter:R,_top:k}}catch(t){
    if(t.message&&t.message.includes("Illegal"))return{language:e,value:V(n),
    illegal:!0,relevance:0,_illegalBy:{message:t.message,index:A,
    context:n.slice(A-100,A+100),mode:t.mode,resultSoFar:O},_emitter:R};if(o)return{
    language:e,value:V(n),illegal:!1,relevance:0,errorRaised:t,_emitter:R,_top:k}
    ;throw t}}function f(e,n){n=n||g.languages||Object.keys(t);const i=(e=>{
    const t={value:V(e),illegal:!1,relevance:0,_top:l,_emitter:new g.__emitter(g)}
    ;return t._emitter.addText(e),t})(e),r=n.filter(E).filter(y).map((t=>h(t,e,!1)))
    ;r.unshift(i);const s=r.sort(((e,t)=>{
    if(e.relevance!==t.relevance)return t.relevance-e.relevance
    ;if(e.language&&t.language){if(E(e.language).supersetOf===t.language)return 1
    ;if(E(t.language).supersetOf===e.language)return-1}return 0})),[o,a]=s,c=o
    ;return c.secondBest=a,c}function p(e){let t=null;const n=(e=>{
    let t=e.className+" ";t+=e.parentNode?e.parentNode.className:""
    ;const n=g.languageDetectRe.exec(t);if(n){const t=E(n[1])
    ;return t||(z(a.replace("{}",n[1])),
    z("Falling back to no-highlight mode for this block.",e)),t?n[1]:"no-highlight"}
    return t.split(/\s+/).find((e=>d(e)||E(e)))})(e);if(d(n))return
    ;w("before:highlightElement",{el:e,language:n
    }),!g.ignoreUnescapedHTML&&e.children.length>0&&(console.warn("One of your code blocks includes unescaped HTML. This is a potentially serious security risk."),
    console.warn("https://github.com/highlightjs/highlight.js/issues/2886"),
    console.warn(e)),t=e;const i=t.textContent,s=n?u(i,{language:n,ignoreIllegals:!0
    }):f(i);e.innerHTML=s.value,((e,t,n)=>{const i=t&&r[t]||n
    ;e.classList.add("hljs"),e.classList.add("language-"+i)
    })(e,n,s.language),e.result={language:s.language,re:s.relevance,
    relevance:s.relevance},s.secondBest&&(e.secondBest={
    language:s.secondBest.language,relevance:s.secondBest.relevance
    }),w("after:highlightElement",{el:e,result:s,text:i})}let b=!1;function m(){
    "loading"!==document.readyState?document.querySelectorAll(g.cssSelector).forEach(p):b=!0
    }function E(e){return e=(e||"").toLowerCase(),t[e]||t[r[e]]}
    function _(e,{languageName:t}){"string"==typeof e&&(e=[e]),e.forEach((e=>{
    r[e.toLowerCase()]=t}))}function y(e){const t=E(e)
    ;return t&&!t.disableAutodetect}function w(e,t){const n=e;s.forEach((e=>{
    e[n]&&e[n](t)}))}
    "undefined"!=typeof window&&window.addEventListener&&window.addEventListener("DOMContentLoaded",(()=>{
    b&&m()}),!1),Object.assign(e,{highlight:u,highlightAuto:f,highlightAll:m,
    highlightElement:p,
    highlightBlock:e=>(K("10.7.0","highlightBlock will be removed entirely in v12.0"),
    K("10.7.0","Please use highlightElement now."),p(e)),configure:e=>{g=q(g,e)},
    initHighlighting:()=>{
    m(),K("10.6.0","initHighlighting() deprecated.  Use highlightAll() now.")},
    initHighlightingOnLoad:()=>{
    m(),K("10.6.0","initHighlightingOnLoad() deprecated.  Use highlightAll() now.")
    },registerLanguage:(n,i)=>{let r=null;try{r=i(e)}catch(e){
    if(H("Language definition for '{}' could not be registered.".replace("{}",n)),
    !o)throw e;H(e),r=l}
    r.name||(r.name=n),t[n]=r,r.rawDefinition=i.bind(null,e),r.aliases&&_(r.aliases,{
    languageName:n})},unregisterLanguage:e=>{delete t[e]
    ;for(const t of Object.keys(r))r[t]===e&&delete r[t]},
    listLanguages:()=>Object.keys(t),getLanguage:E,registerAliases:_,
    autoDetection:y,inherit:q,addPlugin:e=>{(e=>{
    e["before:highlightBlock"]&&!e["before:highlightElement"]&&(e["before:highlightElement"]=t=>{
    e["before:highlightBlock"](Object.assign({block:t.el},t))
    }),e["after:highlightBlock"]&&!e["after:highlightElement"]&&(e["after:highlightElement"]=t=>{
    e["after:highlightBlock"](Object.assign({block:t.el},t))})})(e),s.push(e)}
    }),e.debugMode=()=>{o=!1},e.safeMode=()=>{o=!0},e.versionString="11.1.0"
    ;for(const e in M)"object"==typeof M[e]&&n(M[e]);return Object.assign(e,M),e
    })({}),Y=Object.freeze({__proto__:null,grmr_sway:e=>{const t={
    className:"title.function.invoke",relevance:0,
    begin:u(/\b/,/(?!let\b)/,e.IDENT_RE,d(/\s*\(/))},n="([u](8|16|32|64))?",i=[]
    ;return{name:"Sway",aliases:["sw"],keywords:{$pattern:e.IDENT_RE+"!?",
    type:["u8","u16","u32","u64"],
    keyword:["as","asm","contract","deref","enum","fn","impl","let","library","match","mut","predicate","ref","return","script","Self","self","str","struct","trait","use","where","while"],
    literal:["true","false"],built_in:i},illegal:"</",
    contains:[e.C_LINE_COMMENT_MODE,e.COMMENT("/\\*","\\*/",{contains:["self"]
    }),e.inherit(e.QUOTE_STRING_MODE,{begin:/b?"/,illegal:null}),{
    className:"string",variants:[{begin:/b?r(#*)"(.|\n)*?"\1(?!#)/},{
    begin:/b?'\\?(x\w{2}|u\w{4}|U\w{8}|.)'/}]},{className:"symbol",
    begin:/'[a-zA-Z_][a-zA-Z0-9_]*/},{className:"number",variants:[{
    begin:"\\b0b([01_]+)"+n},{begin:"\\b0o([0-7_]+)"+n},{
    begin:"\\b0x([A-Fa-f0-9_]+)"+n},{
    begin:"\\b(\\d[\\d_]*(\\.[0-9_]+)?([eE][+-]?[0-9_]+)?)"+n}],relevance:0},{
    begin:[/fn/,/\s+/,e.UNDERSCORE_IDENT_RE],className:{1:"keyword",
    3:"title.function"}},{begin:[/let/,/\s+/,/(?:mut\s+)?/,e.UNDERSCORE_IDENT_RE],
    className:{1:"keyword",3:"keyword",4:"variable"}},{
    begin:[/type/,/\s+/,e.UNDERSCORE_IDENT_RE],className:{1:"keyword",
    3:"title.class"}},{
    begin:[/(?:trait|enum|struct|impl|for|library)/,/\s+/,e.UNDERSCORE_IDENT_RE],
    className:{1:"keyword",3:"title.class"}},{begin:e.IDENT_RE+"::",keywords:{
    keyword:"Self",built_in:i}},{className:"punctuation",begin:"->"},t]}}})
    ;const ee=Q;for(const e of Object.keys(Y)){
    const t=e.replace("grmr_","").replace("_","-");ee.registerLanguage(t,Y[e])}
    return ee}()
    ;"object"==typeof exports&&"undefined"!=typeof module&&(module.exports=hljs);