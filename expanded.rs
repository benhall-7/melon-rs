#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
mod melon {
    pub(self) mod sys {
        use autocxx::prelude::*;
        #[allow(non_snake_case)]
        #[allow(dead_code)]
        #[allow(non_upper_case_globals)]
        #[allow(non_camel_case_types)]
        mod ffi {
            pub trait ToCppString {
                fn into_cpp(self) -> cxx::UniquePtr<cxx::CxxString>;
            }
            impl ToCppString for &str {
                fn into_cpp(self) -> cxx::UniquePtr<cxx::CxxString> {
                    make_string(self)
                }
            }
            impl ToCppString for String {
                fn into_cpp(self) -> cxx::UniquePtr<cxx::CxxString> {
                    make_string(&self)
                }
            }
            impl ToCppString for &String {
                fn into_cpp(self) -> cxx::UniquePtr<cxx::CxxString> {
                    make_string(self)
                }
            }
            impl ToCppString for cxx::UniquePtr<cxx::CxxString> {
                fn into_cpp(self) -> cxx::UniquePtr<cxx::CxxString> {
                    self
                }
            }
            unsafe impl cxx::ExternType for bindgen::root::Savestate {
                type Id = (
                    ::cxx::S,
                    ::cxx::a,
                    ::cxx::v,
                    ::cxx::e,
                    ::cxx::s,
                    ::cxx::t,
                    ::cxx::a,
                    ::cxx::t,
                    ::cxx::e,
                );
                type Kind = cxx::kind::Opaque;
            }
            mod bindgen {
                pub(super) mod root {
                    #[repr(C, align(8))]
                    pub struct Savestate {
                        _pinned: core::marker::PhantomData<core::marker::PhantomPinned>,
                        _non_send_sync: core::marker::PhantomData<[*const u8; 0]>,
                        _data: [u8; 24],
                    }
                    pub type u32_ = u32;
                    pub type u8_ = u8;
                    pub type u16_ = u16;
                    pub type u64_ = u64;
                    impl Savestate {
                        pub fn new(
                            filename: impl ToCppString,
                            save: bool,
                        ) -> impl autocxx::moveit::new::New<Output = Self> {
                            unsafe {
                                autocxx::moveit::new::by_raw(move |this| {
                                    let this = this.get_unchecked_mut().as_mut_ptr();
                                    cxxbridge::new_autocxx_autocxx_wrapper(
                                        this,
                                        filename.into_cpp(),
                                        save,
                                    )
                                })
                            }
                        }
                    }
                    unsafe impl autocxx::moveit::MakeCppStorage for root::Savestate {
                        unsafe fn allocate_uninitialized_cpp_storage() -> *mut root::Savestate {
                            cxxbridge::Savestate_alloc_autocxx_wrapper()
                        }
                        unsafe fn free_uninitialized_cpp_storage(
                            arg0: *mut root::Savestate,
                        ) {
                            cxxbridge::Savestate_free_autocxx_wrapper(arg0)
                        }
                    }
                    impl Drop for root::Savestate {
                        fn drop(self: &mut root::Savestate) {
                            unsafe {
                                cxxbridge::Savestate_destructor_autocxx_wrapper(self)
                            }
                        }
                    }
                    unsafe impl autocxx::moveit::new::CopyNew for root::Savestate {
                        ///Synthesized copy constructor.
                        unsafe fn copy_new(
                            other: &root::Savestate,
                            this: ::std::pin::Pin<
                                &mut ::std::mem::MaybeUninit<root::Savestate>,
                            >,
                        ) {
                            cxxbridge::new_synthetic_const_copy_ctor_0x2a17e0bbc16df75_autocxx_wrapper(
                                this.get_unchecked_mut().as_mut_ptr(),
                                other,
                            )
                        }
                    }
                    #[allow(unused_imports)]
                    use self::super::super::{cxxbridge, ToCppString};
                    #[allow(unused_imports)]
                    use self::super::root;
                }
            }
            #[deny(improper_ctypes, improper_ctypes_definitions)]
            #[allow(clippy::unknown_clippy_lints)]
            #[allow(non_camel_case_types, non_snake_case, clippy::upper_case_acronyms)]
            mod cxxbridge {
                pub fn autocxx_make_string_0x2a17e0bbc16df75(
                    str_: &str,
                ) -> ::cxx::UniquePtr<::cxx::CxxString> {
                    extern "C" {
                        #[link_name = "cxxbridge1$autocxx_make_string_0x2a17e0bbc16df75"]
                        fn __autocxx_make_string_0x2a17e0bbc16df75(
                            str_: ::cxx::private::RustStr,
                        ) -> *mut ::cxx::CxxString;
                    }
                    unsafe {
                        ::cxx::UniquePtr::from_raw(
                            __autocxx_make_string_0x2a17e0bbc16df75(
                                ::cxx::private::RustStr::from(str_),
                            ),
                        )
                    }
                }
                pub unsafe fn Savestate_alloc_autocxx_wrapper() -> *mut Savestate {
                    extern "C" {
                        #[link_name = "cxxbridge1$Savestate_alloc_autocxx_wrapper"]
                        fn __Savestate_alloc_autocxx_wrapper() -> *mut ::cxx::core::ffi::c_void;
                    }
                    __Savestate_alloc_autocxx_wrapper().cast()
                }
                pub unsafe fn Savestate_free_autocxx_wrapper(arg0: *mut Savestate) {
                    extern "C" {
                        #[link_name = "cxxbridge1$Savestate_free_autocxx_wrapper"]
                        fn __Savestate_free_autocxx_wrapper(
                            arg0: *mut ::cxx::core::ffi::c_void,
                        );
                    }
                    __Savestate_free_autocxx_wrapper(arg0.cast())
                }
                pub type Savestate = super::bindgen::root::Savestate;
                pub fn Init() -> bool {
                    extern "C" {
                        #[link_name = "NDS$cxxbridge1$Init"]
                        fn __Init() -> bool;
                    }
                    unsafe { __Init() }
                }
                pub fn DeInit() {
                    extern "C" {
                        #[link_name = "NDS$cxxbridge1$DeInit"]
                        fn __DeInit();
                    }
                    unsafe { __DeInit() }
                }
                pub fn Reset() {
                    extern "C" {
                        #[link_name = "NDS$cxxbridge1$Reset"]
                        fn __Reset();
                    }
                    unsafe { __Reset() }
                }
                pub fn Start() {
                    extern "C" {
                        #[link_name = "NDS$cxxbridge1$Start"]
                        fn __Start();
                    }
                    unsafe { __Start() }
                }
                pub fn Stop() {
                    extern "C" {
                        #[link_name = "NDS$cxxbridge1$Stop"]
                        fn __Stop();
                    }
                    unsafe { __Stop() }
                }
                pub unsafe fn DoSavestate(file: *mut Savestate) -> bool {
                    extern "C" {
                        #[link_name = "NDS$cxxbridge1$DoSavestate"]
                        fn __DoSavestate(file: *mut ::cxx::core::ffi::c_void) -> bool;
                    }
                    __DoSavestate(file.cast())
                }
                pub fn SetARM9RegionTimings(
                    addrstart: u32,
                    addrend: u32,
                    region: u32,
                    buswidth: c_int,
                    nonseq: c_int,
                    seq: c_int,
                ) {
                    extern "C" {
                        #[link_name = "NDS$cxxbridge1$SetARM9RegionTimings"]
                        fn __SetARM9RegionTimings(
                            addrstart: u32,
                            addrend: u32,
                            region: u32,
                            buswidth: *mut c_int,
                            nonseq: *mut c_int,
                            seq: *mut c_int,
                        );
                    }
                    unsafe {
                        let mut buswidth = ::cxx::core::mem::MaybeUninit::new(buswidth);
                        let mut nonseq = ::cxx::core::mem::MaybeUninit::new(nonseq);
                        let mut seq = ::cxx::core::mem::MaybeUninit::new(seq);
                        __SetARM9RegionTimings(
                            addrstart,
                            addrend,
                            region,
                            buswidth.as_mut_ptr(),
                            nonseq.as_mut_ptr(),
                            seq.as_mut_ptr(),
                        )
                    }
                }
                pub fn SetARM7RegionTimings(
                    addrstart: u32,
                    addrend: u32,
                    region: u32,
                    buswidth: c_int,
                    nonseq: c_int,
                    seq: c_int,
                ) {
                    extern "C" {
                        #[link_name = "NDS$cxxbridge1$SetARM7RegionTimings"]
                        fn __SetARM7RegionTimings(
                            addrstart: u32,
                            addrend: u32,
                            region: u32,
                            buswidth: *mut c_int,
                            nonseq: *mut c_int,
                            seq: *mut c_int,
                        );
                    }
                    unsafe {
                        let mut buswidth = ::cxx::core::mem::MaybeUninit::new(buswidth);
                        let mut nonseq = ::cxx::core::mem::MaybeUninit::new(nonseq);
                        let mut seq = ::cxx::core::mem::MaybeUninit::new(seq);
                        __SetARM7RegionTimings(
                            addrstart,
                            addrend,
                            region,
                            buswidth.as_mut_ptr(),
                            nonseq.as_mut_ptr(),
                            seq.as_mut_ptr(),
                        )
                    }
                }
                pub fn SetConsoleType(type_: c_int) {
                    extern "C" {
                        #[link_name = "NDS$cxxbridge1$SetConsoleType"]
                        fn __SetConsoleType(type_: *mut c_int);
                    }
                    unsafe {
                        let mut type_ = ::cxx::core::mem::MaybeUninit::new(type_);
                        __SetConsoleType(type_.as_mut_ptr())
                    }
                }
                pub fn LoadBIOS() {
                    extern "C" {
                        #[link_name = "NDS$cxxbridge1$LoadBIOS"]
                        fn __LoadBIOS();
                    }
                    unsafe { __LoadBIOS() }
                }
                pub unsafe fn LoadCart(
                    romdata: *const u8,
                    romlen: u32,
                    savedata: *const u8,
                    savelen: u32,
                ) -> bool {
                    extern "C" {
                        #[link_name = "NDS$cxxbridge1$LoadCart"]
                        fn __LoadCart(
                            romdata: *const u8,
                            romlen: u32,
                            savedata: *const u8,
                            savelen: u32,
                        ) -> bool;
                    }
                    __LoadCart(romdata, romlen, savedata, savelen)
                }
                pub unsafe fn LoadSave(savedata: *const u8, savelen: u32) {
                    extern "C" {
                        #[link_name = "NDS$cxxbridge1$LoadSave"]
                        fn __LoadSave(savedata: *const u8, savelen: u32);
                    }
                    __LoadSave(savedata, savelen)
                }
                pub fn EjectCart() {
                    extern "C" {
                        #[link_name = "NDS$cxxbridge1$EjectCart"]
                        fn __EjectCart();
                    }
                    unsafe { __EjectCart() }
                }
                pub fn CartInserted() -> bool {
                    extern "C" {
                        #[link_name = "NDS$cxxbridge1$CartInserted"]
                        fn __CartInserted() -> bool;
                    }
                    unsafe { __CartInserted() }
                }
                impl Savestate {
                    pub unsafe fn Section(
                        self: ::cxx::core::pin::Pin<&mut Self>,
                        magic: *const ::cxx::private::c_char,
                    ) {
                        extern "C" {
                            #[link_name = "cxxbridge1$Savestate$Section"]
                            fn __Section(
                                _: ::cxx::core::pin::Pin<&mut Savestate>,
                                magic: *const ::cxx::private::c_char,
                            );
                        }
                        __Section(self, magic)
                    }
                }
                impl Savestate {
                    pub unsafe fn Var8(
                        self: ::cxx::core::pin::Pin<&mut Self>,
                        var: *mut u8,
                    ) {
                        extern "C" {
                            #[link_name = "cxxbridge1$Savestate$Var8"]
                            fn __Var8(
                                _: ::cxx::core::pin::Pin<&mut Savestate>,
                                var: *mut u8,
                            );
                        }
                        __Var8(self, var)
                    }
                }
                impl Savestate {
                    pub unsafe fn Var16(
                        self: ::cxx::core::pin::Pin<&mut Self>,
                        var: *mut u16,
                    ) {
                        extern "C" {
                            #[link_name = "cxxbridge1$Savestate$Var16"]
                            fn __Var16(
                                _: ::cxx::core::pin::Pin<&mut Savestate>,
                                var: *mut u16,
                            );
                        }
                        __Var16(self, var)
                    }
                }
                impl Savestate {
                    pub unsafe fn Var32(
                        self: ::cxx::core::pin::Pin<&mut Self>,
                        var: *mut u32,
                    ) {
                        extern "C" {
                            #[link_name = "cxxbridge1$Savestate$Var32"]
                            fn __Var32(
                                _: ::cxx::core::pin::Pin<&mut Savestate>,
                                var: *mut u32,
                            );
                        }
                        __Var32(self, var)
                    }
                }
                impl Savestate {
                    pub unsafe fn Var64(
                        self: ::cxx::core::pin::Pin<&mut Self>,
                        var: *mut u64,
                    ) {
                        extern "C" {
                            #[link_name = "cxxbridge1$Savestate$Var64"]
                            fn __Var64(
                                _: ::cxx::core::pin::Pin<&mut Savestate>,
                                var: *mut u64,
                            );
                        }
                        __Var64(self, var)
                    }
                }
                impl Savestate {
                    pub unsafe fn Bool32(
                        self: ::cxx::core::pin::Pin<&mut Self>,
                        var: *mut bool,
                    ) {
                        extern "C" {
                            #[link_name = "cxxbridge1$Savestate$Bool32"]
                            fn __Bool32(
                                _: ::cxx::core::pin::Pin<&mut Savestate>,
                                var: *mut bool,
                            );
                        }
                        __Bool32(self, var)
                    }
                }
                impl Savestate {
                    pub unsafe fn VarArray(
                        self: ::cxx::core::pin::Pin<&mut Self>,
                        data: *mut c_void,
                        len: u32,
                    ) {
                        extern "C" {
                            #[link_name = "cxxbridge1$Savestate$VarArray"]
                            fn __VarArray(
                                _: ::cxx::core::pin::Pin<&mut Savestate>,
                                data: *mut ::cxx::core::ffi::c_void,
                                len: u32,
                            );
                        }
                        __VarArray(self, data.cast(), len)
                    }
                }
                impl Savestate {
                    pub fn IsAtleastVersion(
                        self: ::cxx::core::pin::Pin<&mut Self>,
                        major: u32,
                        minor: u32,
                    ) -> bool {
                        extern "C" {
                            #[link_name = "cxxbridge1$Savestate$IsAtleastVersion"]
                            fn __IsAtleastVersion(
                                _: ::cxx::core::pin::Pin<&mut Savestate>,
                                major: u32,
                                minor: u32,
                            ) -> bool;
                        }
                        unsafe { __IsAtleastVersion(self, major, minor) }
                    }
                }
                pub unsafe fn new_autocxx_autocxx_wrapper(
                    autocxx_gen_this: *mut Savestate,
                    filename: ::cxx::UniquePtr<::cxx::CxxString>,
                    save: bool,
                ) {
                    extern "C" {
                        #[link_name = "cxxbridge1$new_autocxx_autocxx_wrapper"]
                        fn __new_autocxx_autocxx_wrapper(
                            autocxx_gen_this: *mut ::cxx::core::ffi::c_void,
                            filename: *mut ::cxx::CxxString,
                            save: bool,
                        );
                    }
                    __new_autocxx_autocxx_wrapper(
                        autocxx_gen_this.cast(),
                        ::cxx::UniquePtr::into_raw(filename),
                        save,
                    )
                }
                pub unsafe fn Savestate_destructor_autocxx_wrapper(
                    autocxx_gen_this: *mut Savestate,
                ) {
                    extern "C" {
                        #[link_name = "cxxbridge1$Savestate_destructor_autocxx_wrapper"]
                        fn __Savestate_destructor_autocxx_wrapper(
                            autocxx_gen_this: *mut ::cxx::core::ffi::c_void,
                        );
                    }
                    __Savestate_destructor_autocxx_wrapper(autocxx_gen_this.cast())
                }
                ///Synthesized copy constructor.
                pub unsafe fn new_synthetic_const_copy_ctor_0x2a17e0bbc16df75_autocxx_wrapper(
                    autocxx_gen_this: *mut Savestate,
                    other: &Savestate,
                ) {
                    extern "C" {
                        #[link_name = "cxxbridge1$new_synthetic_const_copy_ctor_0x2a17e0bbc16df75_autocxx_wrapper"]
                        fn __new_synthetic_const_copy_ctor_0x2a17e0bbc16df75_autocxx_wrapper(
                            autocxx_gen_this: *mut ::cxx::core::ffi::c_void,
                            other: *const ::cxx::core::ffi::c_void,
                        );
                    }
                    __new_synthetic_const_copy_ctor_0x2a17e0bbc16df75_autocxx_wrapper(
                        autocxx_gen_this.cast(),
                        other as *const Savestate as *const ::cxx::core::ffi::c_void,
                    )
                }
                pub type c_int = autocxx::c_int;
                pub type c_void = autocxx::c_void;
                unsafe impl ::cxx::private::UniquePtrTarget for Savestate {
                    fn __typename(
                        f: &mut ::cxx::core::fmt::Formatter<'_>,
                    ) -> ::cxx::core::fmt::Result {
                        f.write_str("Savestate")
                    }
                    fn __null() -> ::cxx::core::mem::MaybeUninit<
                        *mut ::cxx::core::ffi::c_void,
                    > {
                        extern "C" {
                            #[link_name = "cxxbridge1$unique_ptr$Savestate$null"]
                            fn __null(
                                this: *mut ::cxx::core::mem::MaybeUninit<
                                    *mut ::cxx::core::ffi::c_void,
                                >,
                            );
                        }
                        let mut repr = ::cxx::core::mem::MaybeUninit::uninit();
                        unsafe { __null(&mut repr) }
                        repr
                    }
                    fn __new(
                        value: Self,
                    ) -> ::cxx::core::mem::MaybeUninit<*mut ::cxx::core::ffi::c_void> {
                        extern "C" {
                            #[link_name = "cxxbridge1$unique_ptr$Savestate$uninit"]
                            fn __uninit(
                                this: *mut ::cxx::core::mem::MaybeUninit<
                                    *mut ::cxx::core::ffi::c_void,
                                >,
                            ) -> *mut ::cxx::core::ffi::c_void;
                        }
                        let mut repr = ::cxx::core::mem::MaybeUninit::uninit();
                        unsafe { __uninit(&mut repr).cast::<Savestate>().write(value) }
                        repr
                    }
                    unsafe fn __raw(
                        raw: *mut Self,
                    ) -> ::cxx::core::mem::MaybeUninit<*mut ::cxx::core::ffi::c_void> {
                        extern "C" {
                            #[link_name = "cxxbridge1$unique_ptr$Savestate$raw"]
                            fn __raw(
                                this: *mut ::cxx::core::mem::MaybeUninit<
                                    *mut ::cxx::core::ffi::c_void,
                                >,
                                raw: *mut ::cxx::core::ffi::c_void,
                            );
                        }
                        let mut repr = ::cxx::core::mem::MaybeUninit::uninit();
                        __raw(&mut repr, raw.cast());
                        repr
                    }
                    unsafe fn __get(
                        repr: ::cxx::core::mem::MaybeUninit<
                            *mut ::cxx::core::ffi::c_void,
                        >,
                    ) -> *const Self {
                        extern "C" {
                            #[link_name = "cxxbridge1$unique_ptr$Savestate$get"]
                            fn __get(
                                this: *const ::cxx::core::mem::MaybeUninit<
                                    *mut ::cxx::core::ffi::c_void,
                                >,
                            ) -> *const ::cxx::core::ffi::c_void;
                        }
                        __get(&repr).cast()
                    }
                    unsafe fn __release(
                        mut repr: ::cxx::core::mem::MaybeUninit<
                            *mut ::cxx::core::ffi::c_void,
                        >,
                    ) -> *mut Self {
                        extern "C" {
                            #[link_name = "cxxbridge1$unique_ptr$Savestate$release"]
                            fn __release(
                                this: *mut ::cxx::core::mem::MaybeUninit<
                                    *mut ::cxx::core::ffi::c_void,
                                >,
                            ) -> *mut ::cxx::core::ffi::c_void;
                        }
                        __release(&mut repr).cast()
                    }
                    unsafe fn __drop(
                        mut repr: ::cxx::core::mem::MaybeUninit<
                            *mut ::cxx::core::ffi::c_void,
                        >,
                    ) {
                        extern "C" {
                            #[link_name = "cxxbridge1$unique_ptr$Savestate$drop"]
                            fn __drop(
                                this: *mut ::cxx::core::mem::MaybeUninit<
                                    *mut ::cxx::core::ffi::c_void,
                                >,
                            );
                        }
                        __drop(&mut repr);
                    }
                }
                unsafe impl ::cxx::private::SharedPtrTarget for Savestate {
                    fn __typename(
                        f: &mut ::cxx::core::fmt::Formatter<'_>,
                    ) -> ::cxx::core::fmt::Result {
                        f.write_str("Savestate")
                    }
                    unsafe fn __null(new: *mut ::cxx::core::ffi::c_void) {
                        extern "C" {
                            #[link_name = "cxxbridge1$shared_ptr$Savestate$null"]
                            fn __null(new: *mut ::cxx::core::ffi::c_void);
                        }
                        __null(new);
                    }
                    unsafe fn __new(value: Self, new: *mut ::cxx::core::ffi::c_void) {
                        extern "C" {
                            #[link_name = "cxxbridge1$shared_ptr$Savestate$uninit"]
                            fn __uninit(
                                new: *mut ::cxx::core::ffi::c_void,
                            ) -> *mut ::cxx::core::ffi::c_void;
                        }
                        __uninit(new).cast::<Savestate>().write(value);
                    }
                    unsafe fn __clone(
                        this: *const ::cxx::core::ffi::c_void,
                        new: *mut ::cxx::core::ffi::c_void,
                    ) {
                        extern "C" {
                            #[link_name = "cxxbridge1$shared_ptr$Savestate$clone"]
                            fn __clone(
                                this: *const ::cxx::core::ffi::c_void,
                                new: *mut ::cxx::core::ffi::c_void,
                            );
                        }
                        __clone(this, new);
                    }
                    unsafe fn __get(
                        this: *const ::cxx::core::ffi::c_void,
                    ) -> *const Self {
                        extern "C" {
                            #[link_name = "cxxbridge1$shared_ptr$Savestate$get"]
                            fn __get(
                                this: *const ::cxx::core::ffi::c_void,
                            ) -> *const ::cxx::core::ffi::c_void;
                        }
                        __get(this).cast()
                    }
                    unsafe fn __drop(this: *mut ::cxx::core::ffi::c_void) {
                        extern "C" {
                            #[link_name = "cxxbridge1$shared_ptr$Savestate$drop"]
                            fn __drop(this: *mut ::cxx::core::ffi::c_void);
                        }
                        __drop(this);
                    }
                }
                unsafe impl ::cxx::private::WeakPtrTarget for Savestate {
                    fn __typename(
                        f: &mut ::cxx::core::fmt::Formatter<'_>,
                    ) -> ::cxx::core::fmt::Result {
                        f.write_str("Savestate")
                    }
                    unsafe fn __null(new: *mut ::cxx::core::ffi::c_void) {
                        extern "C" {
                            #[link_name = "cxxbridge1$weak_ptr$Savestate$null"]
                            fn __null(new: *mut ::cxx::core::ffi::c_void);
                        }
                        __null(new);
                    }
                    unsafe fn __clone(
                        this: *const ::cxx::core::ffi::c_void,
                        new: *mut ::cxx::core::ffi::c_void,
                    ) {
                        extern "C" {
                            #[link_name = "cxxbridge1$weak_ptr$Savestate$clone"]
                            fn __clone(
                                this: *const ::cxx::core::ffi::c_void,
                                new: *mut ::cxx::core::ffi::c_void,
                            );
                        }
                        __clone(this, new);
                    }
                    unsafe fn __downgrade(
                        shared: *const ::cxx::core::ffi::c_void,
                        weak: *mut ::cxx::core::ffi::c_void,
                    ) {
                        extern "C" {
                            #[link_name = "cxxbridge1$weak_ptr$Savestate$downgrade"]
                            fn __downgrade(
                                shared: *const ::cxx::core::ffi::c_void,
                                weak: *mut ::cxx::core::ffi::c_void,
                            );
                        }
                        __downgrade(shared, weak);
                    }
                    unsafe fn __upgrade(
                        weak: *const ::cxx::core::ffi::c_void,
                        shared: *mut ::cxx::core::ffi::c_void,
                    ) {
                        extern "C" {
                            #[link_name = "cxxbridge1$weak_ptr$Savestate$upgrade"]
                            fn __upgrade(
                                weak: *const ::cxx::core::ffi::c_void,
                                shared: *mut ::cxx::core::ffi::c_void,
                            );
                        }
                        __upgrade(weak, shared);
                    }
                    unsafe fn __drop(this: *mut ::cxx::core::ffi::c_void) {
                        extern "C" {
                            #[link_name = "cxxbridge1$weak_ptr$Savestate$drop"]
                            fn __drop(this: *mut ::cxx::core::ffi::c_void);
                        }
                        __drop(this);
                    }
                }
                #[doc(hidden)]
                const _: () = {
                    const _: fn() = ::cxx::private::verify_extern_type::<
                        Savestate,
                        (
                            ::cxx::S,
                            ::cxx::a,
                            ::cxx::v,
                            ::cxx::e,
                            ::cxx::s,
                            ::cxx::t,
                            ::cxx::a,
                            ::cxx::t,
                            ::cxx::e,
                        ),
                    >;
                    const _: fn() = ::cxx::private::verify_extern_type::<
                        c_int,
                        (::cxx::c, ::cxx::__, ::cxx::i, ::cxx::n, ::cxx::t),
                    >;
                    const _: fn() = ::cxx::private::verify_extern_kind::<
                        c_int,
                        ::cxx::kind::Trivial,
                    >;
                    const _: fn() = ::cxx::private::verify_extern_type::<
                        c_void,
                        (::cxx::c, ::cxx::__, ::cxx::v, ::cxx::o, ::cxx::i, ::cxx::d),
                    >;
                };
            }
            #[allow(unused_imports)]
            use bindgen::root;
            pub use cxxbridge::autocxx_make_string_0x2a17e0bbc16df75 as make_string;
            pub use bindgen::root::Savestate;
            pub use bindgen::root::u32_;
            pub use bindgen::root::u8_;
            pub use bindgen::root::u16_;
            pub use bindgen::root::u64_;
            pub mod NDS {
                pub use super::cxxbridge::Init;
                pub use super::cxxbridge::DeInit;
                pub use super::cxxbridge::Reset;
                pub use super::cxxbridge::Start;
                pub use super::cxxbridge::Stop;
                pub use super::cxxbridge::DoSavestate;
                pub use super::cxxbridge::SetARM9RegionTimings;
                pub use super::cxxbridge::SetARM7RegionTimings;
                pub use super::cxxbridge::SetConsoleType;
                pub use super::cxxbridge::LoadBIOS;
                pub use super::cxxbridge::LoadCart;
                pub use super::cxxbridge::LoadSave;
                pub use super::cxxbridge::EjectCart;
                pub use super::cxxbridge::CartInserted;
            }
        }
        pub use ffi::*;
    }
    pub mod nds {
        use super::sys;
        pub fn cart_inserted() -> bool {
            sys::NDS::CartInserted()
        }
    }
}
fn main() {
    {
        ::std::io::_print(
            ::core::fmt::Arguments::new_v1(
                &["", "\n"],
                &[::core::fmt::ArgumentV1::new_display(&melon::nds::cart_inserted())],
            ),
        );
    }
}
