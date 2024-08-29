script;

mod lib;

// module const imported more then once
use lib::L_A;
use lib::L_A;

// module const shadowing an imported const
use lib::L_X;
const L_X = 1; 

// module const shadowing a module const
const M_X = 2;
const M_X = 3;

const M_Y = 4;

const M_Z = 41;

use lib::L_Y;
use lib::L_Z;

use lib::L_Z as L_Z_ALIAS;

struct StructWithConstNames {
    M_X: u64,
    L_Y: u64,
    L_Z_ALIAS: u64,
}

fn main() {
    // local const shadowing a module const
    const M_Y = 5;
    {
        const M_Y = 55; // no error message here
    }

    // local const shadowing a const imported in module
    const L_Y = 6;

    // local const shadowing a local const
    const F_X = 7;
    const F_X = 8;
    {
        const F_X = 81; // no error message here
    }

    {
        // scoped local const shadowing a scoped local const
        const F_Y = 9;
        {
            const F_Y = 10;
        }
    }

    // variable shadowing an imported const
    let L_X = 100;
    {
        let L_X = 101; // no error message here
    }

    // variable shadowing a local const
    const F_Z = 11;
    let F_Z = 102;

    // local const shadowing a variable
    let F_A = 103;
    const F_A = 12;

    // variable shadowing a variable - this is okay!
    let A = 104;
    let A = 105;

    // variable shadowing a module const
    let M_Y = 106;

    {
        // scoped variable shadowing a module const
        let M_Z = 107;
    }

    let B = 108;
    {
        // scoped const shadowing a variable
        const B = 13;
    }

    const F_K = 14;
    {
        // scoped variable shadowing a local const
        let F_K = 109;
    }

    {
        // scoped variable shadowing imported const
        let L_Z = 110;
    }

    // variable shadowing a locally imported const
    use lib::L_K;
    let L_K = 111;

    // const shadowing a locally imported const
    use lib::L_M;
    const L_M = 15;

    let s = StructWithConstNames {
        M_X,
        L_Y,
        L_Z_ALIAS,
    };

    // pattern variables shadowing different types of consts
    let _ = match s {
        StructWithConstNames { M_X, L_Y, L_Z_ALIAS } => {
            42
        },
    };
}

use lib::L_N;

const M_M = 16;

struct S { }

impl S {
    const S_X = 200;
    const S_X = 201;

    const L_N = 202;

    const M_M = 203;

    const S_Y = 204;

    fn f() {
        const S_Y = 205;
    }
}

enum E { }

impl E {
    const E_X = 300;
    const E_X = 301;

    const L_N = 302;

    const M_M = 303;

    const E_Y = 304;

    fn f() {
        const E_Y = 305;
    }
}
