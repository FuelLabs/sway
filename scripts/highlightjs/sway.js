/*
Language: Sway
Author: Fuel Labs contact@fuel.sh
Contributors: Fuel Labs <contact@fuel.sh>,
Website: https://fuel.sh
Category: smart contracts, layer 2, blockchain,
*/

import * as regex from '../lib/regex.js';

/** @type LanguageFn */
export default function(hljs) {
  const FUNCTION_INVOKE = {
    className: "title.function.invoke",
    relevance: 0,
    begin: regex.concat(
      /\b/,
      /(?!let\b)/,
      hljs.IDENT_RE,
      regex.lookahead(/\s*\(/))
  };
  const NUMBER_SUFFIX = '([u](8|16|32|64))\?';

  const KEYWORDS = [
    "abi",
    "as",
    "asm",
    "const",
    "contract",
    "deref",
    "enum",
    "fn",
    "if",
    "impl",
    "let",
    "library",
    "match",
    "mut",
    "else",
    "predicate",
    "ref",
    "return",
    "script",
    "Self",
    "self",
    "storage",
    "str",
    "struct",
    "trait",
    "use",
    "where",
    "while",
  ];
  const LITERALS = [
    "true",
    "false",
  ];
  const BUILTINS = [
    
  ];
  const TYPES = [
    "bool", "char", "u8", "u16", "u32", "u64", "b256", "str", "Self"
  ];
  return {
    name: 'Sway',
    aliases: [ 'sw' ],
    keywords: {
      $pattern: hljs.IDENT_RE + '!?',
      keyword: KEYWORDS,
      literal: LITERALS,
      built_in: TYPES
    },
    illegal: '</',
    contains: [
      hljs.C_LINE_COMMENT_MODE,
      hljs.COMMENT('/\\*', '\\*/', {
        contains: [ 'self' ]
      }),
      hljs.inherit(hljs.QUOTE_STRING_MODE, {
        begin: /b?"/,
        illegal: null
      }),
      {
        className: 'string',
        variants: [
          {
            begin: /b?r(#*)"(.|\n)*?"\1(?!#)/
          },
          {
            begin: /b?'\\?(x\w{2}|u\w{4}|U\w{8}|.)'/
          }
        ]
      },
      {
        className: 'symbol',
        begin: /'[a-zA-Z_][a-zA-Z0-9_]*/
      },
      {
        className: 'number',
        variants: [
          {
            begin: '\\b0b([01_]+)' + NUMBER_SUFFIX
          },
          {
            begin: '\\b0o([0-7_]+)' + NUMBER_SUFFIX
          },
          {
            begin: '\\b0x([A-Fa-f0-9_]+)' + NUMBER_SUFFIX
          },
          {
            begin: '\\b(\\d[\\d_]*(\\.[0-9_]+)?([eE][+-]?[0-9_]+)?)' +
                   NUMBER_SUFFIX
          }
        ],
        relevance: 0
      },
      {
        begin: [
          /fn/,
          /\s+/,
          hljs.UNDERSCORE_IDENT_RE
        ],
        className: {
          1: "keyword",
          3: "title.function"
        }
      },
      {
        begin: [
          /(let|const)/, /\s+/,
          /(?:mut\s+)?/,
          hljs.UNDERSCORE_IDENT_RE
        ],
        className: {
          1: "keyword",
          3: "keyword",
          4: "variable"
        }
      },
      {
        begin: [
          /type/,
          /\s+/,
          hljs.UNDERSCORE_IDENT_RE
        ],
        className: {
          1: "keyword",
          3: "title.class"
        }
      },
      {
        begin: [
          /(?:trait|enum|struct|impl|for|library|abi)/,
          /\s+/,
          hljs.UNDERSCORE_IDENT_RE
        ],
        className: {
          1: "keyword",
          3: "title.class"
        }
      },
      {
        begin: hljs.IDENT_RE + '::',
        keywords: {
          keyword: "Self",
          built_in: BUILTINS
        }
      },
      {
        className: "punctuation",
        begin: '->'
      },
      FUNCTION_INVOKE
    ]
  };
}