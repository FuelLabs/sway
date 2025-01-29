#! /bin/bash

# Needs to exist at least one line between them
remove_generated_code() {
    START=`grep -n "BEGIN $1" ./src/codec.sw`
    START=${START%:*}
    END=`grep -n "END $1" ./src/codec.sw`
    END=${END%:*}
    sed -i "$((START+1)),$((END-1))d" ./src/codec.sw
}

remove_generated_code "ARRAY_ENCODE"
START=1
END=64
for ((i=END;i>=START;i--)); do
    CODE="impl<T> AbiEncode for [T; $i] where T: AbiEncode { fn abi_encode(self, buffer: Buffer) -> Buffer { let mut buffer = buffer; let mut i = 0; while i < $i { buffer = self[i].abi_encode(buffer); i += 1; }; buffer } }"
    sed -i "s/\/\/ BEGIN ARRAY_ENCODE/\/\/ BEGIN ARRAY_ENCODE\n$CODE/g" ./src/codec.sw
done

remove_generated_code "ARRAY_DECODE"
START=1
END=64
for ((i=END;i>=START;i--)); do
    CODE="impl<T> AbiDecode for [T; $i] where T: AbiDecode { fn abi_decode(ref mut buffer: BufferReader) -> [T; $i] { let first: T = buffer.decode::<T>(); let mut array = [first; $i]; let mut i = 1; while i < $i { array[i] = buffer.decode::<T>(); i += 1; }; array } }"
    sed -i "s/\/\/ BEGIN ARRAY_DECODE/\/\/ BEGIN ARRAY_DECODE\n$CODE/g" ./src/codec.sw
done

remove_generated_code "STRARRAY_ENCODE"
START=1
END=64
for ((i=END;i>=START;i--)); do
    CODE="impl AbiEncode for str[$i] { fn abi_encode(self, buffer: Buffer) -> Buffer { Buffer { buffer: __encode_buffer_append(buffer.buffer, self) } } }"
    sed -i "s/\/\/ BEGIN STRARRAY_ENCODE/\/\/ BEGIN STRARRAY_ENCODE\n$CODE/g" ./src/codec.sw
done

remove_generated_code "STRARRAY_DECODE"
START=1
END=64
for ((i=END;i>=START;i--)); do
    CODE="impl AbiDecode for str[$i] { fn abi_decode(ref mut buffer: BufferReader) -> str[$i] { let data = buffer.read_bytes($i); asm(s: data.ptr()) { s: str[$i] } } }"
    sed -i "s/\/\/ BEGIN STRARRAY_DECODE/\/\/ BEGIN STRARRAY_DECODE\n$CODE/g" ./src/codec.sw
done

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

    CODE="$CODE{ fn abi_encode(self, buffer: Buffer) -> Buffer { "

    i=0
    for element in ${elements[@]}
    do
        CODE="$CODE let buffer = self.$i.abi_encode(buffer);"
        i=$((i+1))
    done

    CODE="$CODE buffer } }"

    sed -i "s/\/\/ BEGIN TUPLES_ENCODE/\/\/ BEGIN TUPLES_ENCODE\n$CODE/g" ./src/codec.sw
}

remove_generated_code "TUPLES_ENCODE"
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

    CODE="$CODE{ fn abi_decode(ref mut buffer: BufferReader) -> Self { ("

    for element in ${elements[@]}
    do
        CODE="$CODE $element::abi_decode(buffer),"
    done

    CODE="$CODE) } }"

    sed -i "s/\/\/ BEGIN TUPLES_DECODE/\/\/ BEGIN TUPLES_DECODE\n$CODE/g" ./src/codec.sw
}

remove_generated_code "TUPLES_DECODE"
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
