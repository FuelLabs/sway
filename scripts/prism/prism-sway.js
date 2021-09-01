(function (Prism) {

var multilineComment = /\/\*(?:[^*/]|\*(?!\/)|\/(?!\*)|<self>)*\*\//.source;
for (var i = 0; i < 2; i++) {
    // support 4 levels of nested comments
    multilineComment = multilineComment.replace(/<self>/g, function () { return multilineComment; });
}
multilineComment = multilineComment.replace(/<self>/g, function () { return /[^\s\S]/.source; });


Prism.languages.sway = {
    'comment': [
        {
            pattern: RegExp(/(^|[^\\])/.source + multilineComment),
            lookbehind: true,
            greedy: true
        },
        {
            pattern: /(^|[^\\:])\/\/.*/,
            lookbehind: true,
            greedy: true
        }
    ],
    'string': {
        pattern: /b?"(?:\\[\s\S]|[^\\"])*"|b?r(#*)"(?:[^"]|"(?!\1))*"\1/,
        greedy: true
    },
    'char': {
        pattern: /b?'(?:\\(?:x[0-7][\da-fA-F]|u\{(?:[\da-fA-F]_*){1,6}\}|.)|[^\\\r\n\t'])'/,
        greedy: true,
        alias: 'string'
    },
    'attribute': {
        pattern: /#!?\[(?:[^\[\]"]|"(?:\\[\s\S]|[^\\"])*")*\]/,
        greedy: true,
        alias: 'attr-name',
        inside: {
            'string': null // see below
        }
    },

    // Closure params should not be confused with bitwise OR |
    'closure-params': {
        pattern: /([=(,:]\s*|\bmove\s*)\|[^|]*\||\|[^|]*\|(?=\s*(?:\{|->))/,
        lookbehind: true,
        greedy: true,
        inside: {
            'closure-punctuation': {
                pattern: /^\||\|$/,
                alias: 'punctuation'
            },
            rest: null // see below
        }
    },

    'fragment-specifier': {
        pattern: /(\$\w+:)[a-z]+/,
        lookbehind: true,
        alias: 'punctuation'
    },
    'variable': /\$\w+/,

    'function-definition': {
        pattern: /(\bfn\s+)\w+/,
        lookbehind: true,
        alias: 'function'
    },
    'type-definition': {
        pattern: /(\b(?:enum|struct)\s+)\w+/,
        lookbehind: true,
        alias: 'class-name'
    },
    'module-declaration': [
        {
            pattern: /(\b(?:mod|script|contract|predicate|library)\s+)[a-z][a-z_\d]*/,
            lookbehind: true,
            alias: 'namespace'
        },
        {
            pattern: /(\b(?:crate|self|super)\s*)::\s*[a-z][a-z_\d]*\b(?:\s*::(?:\s*[a-z][a-z_\d]*\s*::)*)?/,
            lookbehind: true,
            alias: 'namespace',
            inside: {
                'punctuation': /::/
            }
        }
    ],
    'keyword': [
        /\b(?:as|break|contract|const|continue|do|else|enum|fn|for|if|impl|in|let|library|match|mod|mut|predicate|priv|pub|ref|return|script|self|Self|static|struct|trait|type|unsized|use|where)\b/,
        // primitives and str
        /\b(?:[u](?:8|16|32|64|128|size)|f(?:32|64)|bool|char|str)\b/
    ],

    // functions can technically start with an upper-case letter, but this will introduce a lot of false positives
    // and Sway's naming conventions recommend snake_case anyway.    
    'function': /\b[a-z_]\w*(?=\s*(?:::\s*<|\())/,
    'macro': {
        pattern: /\b\w+!/,
        alias: 'property'
    },
    'constant': /\b[A-Z_][A-Z_\d]+\b/,
    'class-name': /\b[A-Z]\w*\b/,

    'namespace': {
        pattern: /(?:\b[a-z][a-z_\d]*\s*::\s*)*\b[a-z][a-z_\d]*\s*::(?!\s*<)/,
        inside: {
            'punctuation': /::/
        }
    },

    // Hex, oct, bin, dec numbers with visual separators and type suffix
    'number': /\b(?:0x[\dA-Fa-f](?:_?[\dA-Fa-f])*|0o[0-7](?:_?[0-7])*|0b[01](?:_?[01])*|(?:(?:\d(?:_?\d)*)?\.)?\d(?:_?\d)*(?:[Ee][+-]?\d+)?)(?:_?(?:[u](?:8|16|32|64|size)?|f32|f64))?\b/,
    'boolean': /\b(?:false|true)\b/,
    'punctuation': /->|\.\.=|\.{1,3}|::|[{}[\];(),:]/,
    'operator': /[-+*\/%!^]=?|=[=>]?|&[&=]?|\|[|=]?|<<?=?|>>?=?|[@?]/
};

Prism.languages.sway['closure-params'].inside.rest = Prism.languages.sway;
Prism.languages.sway['attribute'].inside['string'] = Prism.languages.sway['string'];

}(Prism));
