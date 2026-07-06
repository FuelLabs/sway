#! /bin/bash

# Use GNU sed (`gsed` on macOS via `brew install gnu-sed`); BSD sed in-place
# semantics differ and break the substitutions below.
SED=${SED:-sed}
if ! $SED --version >/dev/null 2>&1; then
    if command -v gsed >/dev/null 2>&1; then
        SED=gsed
    else
        echo "GNU sed is required (install with 'brew install gnu-sed' on macOS)." >&2
        exit 1
    fi
fi

# Needs to exist at least one line between them
remove_generated_code() {
    START=`grep -n "BEGIN $1" ./src/$2`
    START=${START%:*}
    END=`grep -n "END $1" ./src/$2`
    END=${END%:*}
    $SED -i "$((START+1)),$((END-1))d" ./src/$2
}

generate_tuple_encode() {
    local CODE="impl<"

    local elements=("$1")
    for element in ${elements[@]}
    do
        CODE="$CODE $element,"
    done

    CODE="$CODE> AbiEncode for ("

    for element in ${elements[@]}
    do
        CODE="$CODE $element,"
    done

    CODE="$CODE) where " 

    for element in ${elements[@]}
    do
        CODE="$CODE $element: AbiEncode, "
    done

    # Emit the body as a sequence of `let r = r && ...;` statements rather
    # than one giant left-deep `&&` chain. Long chains produce a deeply
    # left-leaning AST that drives recursive compiler / LSP transforms into
    # stack overflows on real-world tuples.
    ISTRIVIAL=""
    for element in ${elements[@]}
    do
        ISTRIVIAL="$ISTRIVIAL let r = r \&\& is_encode_trivial::<$element>();"
    done

    CODE="$CODE{ fn is_encode_trivial() -> bool { let r = __runtime_mem_id::<Self>() == __encoding_mem_id::<Self>(); $ISTRIVIAL r } fn abi_encode(self, buffer: Buffer) -> Buffer { "

    i=0
    for element in ${elements[@]}
    do
        CODE="$CODE let buffer = self.$i.abi_encode(buffer);"
        i=$((i+1))
    done

    CODE="$CODE buffer } }"

    $SED -i "s/\/\/ BEGIN TUPLES_ENCODE/\/\/ BEGIN TUPLES_ENCODE\n$CODE/g" ./src/codec.sw
}

remove_generated_code "TUPLES_ENCODE" "codec.sw"
generate_tuple_encode "A B C D E F G H I J K L M N O P Q R S T U V W X Y Z"
generate_tuple_encode "A B C D E F G H I J K L M N O P Q R S T U V W X Y"
generate_tuple_encode "A B C D E F G H I J K L M N O P Q R S T U V W X"
generate_tuple_encode "A B C D E F G H I J K L M N O P Q R S T U V W"
generate_tuple_encode "A B C D E F G H I J K L M N O P Q R S T U V"
generate_tuple_encode "A B C D E F G H I J K L M N O P Q R S T U"
generate_tuple_encode "A B C D E F G H I J K L M N O P Q R S T"
generate_tuple_encode "A B C D E F G H I J K L M N O P Q R S"
generate_tuple_encode "A B C D E F G H I J K L M N O P Q R"
generate_tuple_encode "A B C D E F G H I J K L M N O P Q"
generate_tuple_encode "A B C D E F G H I J K L M N O P"
generate_tuple_encode "A B C D E F G H I J K L M N O"
generate_tuple_encode "A B C D E F G H I J K L M N"
generate_tuple_encode "A B C D E F G H I J K L M"
generate_tuple_encode "A B C D E F G H I J K L"
generate_tuple_encode "A B C D E F G H I J K"
generate_tuple_encode "A B C D E F G H I J"
generate_tuple_encode "A B C D E F G H I"
generate_tuple_encode "A B C D E F G H"
generate_tuple_encode "A B C D E F G"
generate_tuple_encode "A B C D E F"
generate_tuple_encode "A B C D E"
generate_tuple_encode "A B C D"
generate_tuple_encode "A B C"
generate_tuple_encode "A B"
generate_tuple_encode "A"

generate_tuple_decode() {
    local CODE="impl<"

    local elements=("$1")
    for element in ${elements[@]}
    do
        CODE="$CODE $element,"
    done

    CODE="$CODE> AbiDecode for ("

    for element in ${elements[@]}
    do
        CODE="$CODE $element,"
    done

    CODE="$CODE) where " 

    for element in ${elements[@]}
    do
        CODE="$CODE $element: AbiDecode, "
    done

    # Emit the body as a sequence of `let r = r && ...;` statements rather
    # than one giant left-deep `&&` chain. Long chains produce a deeply
    # left-leaning AST that drives recursive compiler / LSP transforms into
    # stack overflows on real-world tuples.
    ISTRIVIAL=""
    for element in ${elements[@]}
    do
        ISTRIVIAL="$ISTRIVIAL let r = r \&\& is_decode_trivial::<$element>();"
    done

    CODE="$CODE{ fn is_decode_trivial() -> bool { let r = __runtime_mem_id::<Self>() == __encoding_mem_id::<Self>(); $ISTRIVIAL r } fn abi_decode(ref mut buffer: BufferReader) -> Self { ("

    for element in ${elements[@]}
    do
        CODE="$CODE $element::abi_decode(buffer),"
    done

    CODE="$CODE) } }"

    $SED -i "s/\/\/ BEGIN TUPLES_DECODE/\/\/ BEGIN TUPLES_DECODE\n$CODE/g" ./src/codec.sw
}

remove_generated_code "TUPLES_DECODE" "codec.sw"
generate_tuple_decode "A B C D E F G H I J K L M N O P Q R S T U V W X Y Z"
generate_tuple_decode "A B C D E F G H I J K L M N O P Q R S T U V W X Y"
generate_tuple_decode "A B C D E F G H I J K L M N O P Q R S T U V W X"
generate_tuple_decode "A B C D E F G H I J K L M N O P Q R S T U V W"
generate_tuple_decode "A B C D E F G H I J K L M N O P Q R S T U V"
generate_tuple_decode "A B C D E F G H I J K L M N O P Q R S T U"
generate_tuple_decode "A B C D E F G H I J K L M N O P Q R S T"
generate_tuple_decode "A B C D E F G H I J K L M N O P Q R S"
generate_tuple_decode "A B C D E F G H I J K L M N O P Q R"
generate_tuple_decode "A B C D E F G H I J K L M N O P Q"
generate_tuple_decode "A B C D E F G H I J K L M N O P"
generate_tuple_decode "A B C D E F G H I J K L M N O"
generate_tuple_decode "A B C D E F G H I J K L M N"
generate_tuple_decode "A B C D E F G H I J K L M"
generate_tuple_decode "A B C D E F G H I J K L"
generate_tuple_decode "A B C D E F G H I J K"
generate_tuple_decode "A B C D E F G H I J"
generate_tuple_decode "A B C D E F G H I"
generate_tuple_decode "A B C D E F G H"
generate_tuple_decode "A B C D E F G"
generate_tuple_decode "A B C D E F"
generate_tuple_decode "A B C D E"
generate_tuple_decode "A B C D"
generate_tuple_decode "A B C"
generate_tuple_decode "A B"
generate_tuple_decode "A"

generate_tuple_debug() {
    local CODE="impl<"

    local elements=("$1")
    for element in ${elements[@]}
    do
        CODE="$CODE $element,"
    done

    CODE="$CODE> Debug for ("

    for element in ${elements[@]}
    do
        CODE="$CODE $element,"
    done

    CODE="$CODE) where " 

    for element in ${elements[@]}
    do
        CODE="$CODE $element: Debug, "
    done

    CODE="$CODE{ fn fmt(self, ref mut f: Formatter) { let mut f = f.debug_tuple(\"\");"

    i=0
    for element in ${elements[@]}
    do
        CODE="$CODE let mut f = f.field(self.$i);"
        i=$((i+1))
    done

    CODE="$CODE f.finish(); } }"

    $SED -i "s/\/\/ BEGIN TUPLES_DEBUG/\/\/ BEGIN TUPLES_DEBUG\n$CODE/g" ./src/debug.sw
}

remove_generated_code "TUPLES_DEBUG" "debug.sw"
generate_tuple_debug "A B C D E F G H I J K L M N O P Q R S T U V W X Y Z"
generate_tuple_debug "A B C D E F G H I J K L M N O P Q R S T U V W X Y"
generate_tuple_debug "A B C D E F G H I J K L M N O P Q R S T U V W X"
generate_tuple_debug "A B C D E F G H I J K L M N O P Q R S T U V W"
generate_tuple_debug "A B C D E F G H I J K L M N O P Q R S T U V"
generate_tuple_debug "A B C D E F G H I J K L M N O P Q R S T U"
generate_tuple_debug "A B C D E F G H I J K L M N O P Q R S T"
generate_tuple_debug "A B C D E F G H I J K L M N O P Q R S"
generate_tuple_debug "A B C D E F G H I J K L M N O P Q R"
generate_tuple_debug "A B C D E F G H I J K L M N O P Q"
generate_tuple_debug "A B C D E F G H I J K L M N O P"
generate_tuple_debug "A B C D E F G H I J K L M N O"
generate_tuple_debug "A B C D E F G H I J K L M N"
generate_tuple_debug "A B C D E F G H I J K L M"
generate_tuple_debug "A B C D E F G H I J K L"
generate_tuple_debug "A B C D E F G H I J K"
generate_tuple_debug "A B C D E F G H I J"
generate_tuple_debug "A B C D E F G H I"
generate_tuple_debug "A B C D E F G H"
generate_tuple_debug "A B C D E F G"
generate_tuple_debug "A B C D E F"
generate_tuple_debug "A B C D E"
generate_tuple_debug "A B C D"
generate_tuple_debug "A B C"
generate_tuple_debug "A B"
generate_tuple_debug "A"

cargo r -p forc-fmt --release -- -p .
