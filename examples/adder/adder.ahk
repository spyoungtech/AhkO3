; In the future, the goal is that all this would be code-generated in addition to the DLL

#DllLoad "*i target\x86_64-pc-windows-gnu\debug\adder.dll"
if !DllCall("GetModuleHandle", "str", "adder") {
    throw Error("Cannot load adder.dll -- please ensure it is on PATH or use #DllLoad to load it in your script before your #Inlude of adder.ahk")
}



concatenate(a, b) {
    res := DllCall("adder\gen_concatenate", "Str", a, "Str", b, "Ptr")
    s := StrGet(res)
    DllCall("adder\ahko3_free_string_ptr", "Ptr", res, "Int64")
    return s
}


add(a, b) {
    res := DllCall("adder\gen_add", "Int64", a, "Int64", b, "Int64")
    return res
}



