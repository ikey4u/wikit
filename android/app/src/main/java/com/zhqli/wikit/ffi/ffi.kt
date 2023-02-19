package com.zhqli.wikit.ffi

class RustBindings {
    private external fun greeting(pattern: String): String

    init {
        System.loadLibrary("native")
    }

    fun sayHello(to: String): String {
        return greeting(to)
    }
}
