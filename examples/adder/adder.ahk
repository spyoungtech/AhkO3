; In the future, the goal is that all this would be code-generated in addition to the DLL

#DllLoad "*i adder.dll"
if !DllCall("GetModuleHandle", "str", "adder") {
    throw Error("Cannot load adder.dll -- please ensure it is on PATH or use #DllLoad to load it in your script before your #Inlude of adder.ahk")
}



concatenate(a, b) {
    res := DllCall("adder\adder_concatenate", "Str", a, "Str", b, "Ptr")
    s := StrGet(res)
    DllCall("adder\ahko3_free_string_ptr", "Ptr", res, "Int64")
    return s
}


add(a, b) {
    res := DllCall("adder\adder_add", "Int64", a, "Int64", b, "Int64")
    return res
}

