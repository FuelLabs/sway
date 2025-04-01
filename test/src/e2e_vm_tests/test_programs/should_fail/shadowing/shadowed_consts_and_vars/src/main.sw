script;

mod lib;

// module const imported more then once
use lib::L_A;
use lib::L_A;

// module const shadowing an imported const
use lib::L_X;
const L_X: u64 = 1; 

// module const shadowing a module const
const M_X: u64 = 2;
const M_X: u64 = 3;

const M_Y: u64 = 4;

const M_Z: u64 = 41;

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
    const M_Y: u64 = 5;
    {
        const M_Y: u64 = 55;
    }

    // local const shadowing a const imported in module
    const L_Y: u64 = 6;

    // local const shadowing a local const
    const F_X: u64 = 7;
    const F_X: u64 = 8;
    {
        const F_X: u64 = 81;
    }

    {
        // scoped local const shadowing a scoped local const
        const F_Y: u64 = 9;
        {
            const F_Y: u64 = 10;
        }
    }

    // variable shadowing an imported const
    let L_X = 100;
    {
        let L_X = 101; // no error message here
    }

    // variable shadowing a local const
    const F_Z: u64 = 11;
    let F_Z = 102;

    // local const shadowing a variable
    let F_A = 103;
    const F_A: u64 = 12;

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
        const B: u64 = 13;
    }

    const F_K: u64 = 14;
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
    const L_M: u64 = 15;

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

const M_M: u64 = 16;

struct S { }

impl S {
    const S_X: u64 = 200;
    const S_X: u64 = 201;

    const L_N: u64 = 202;

    const M_M: u64 = 203;

    const S_Y: u64 = 204;

    fn f() {
        const S_Y: u64 = 205;
    }
}

enum E { }

impl E {
    const E_X: u64 = 300;
    const E_X: u64 = 301;

    const L_N: u64 = 302;

    const M_M: u64 = 303;

    const E_Y: u64 = 304;

    fn f() {
        const E_Y: u64 = 305;
    }
}
