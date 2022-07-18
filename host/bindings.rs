#[allow(clippy::all)]
pub mod wapc_guest {
    #[allow(unused_imports)]
    use wit_bindgen_wasmtime::{anyhow, wasmtime};
    #[derive(Debug)]
    pub struct Wapc(wit_bindgen_wasmtime::rt::ResourceIndex);

    /// Auxiliary data associated with the wasm exports.
    ///
    /// This is required to be stored within the data of a
    /// `Store<T>` itself so lifting/lowering state can be managed
    /// when translating between the host and wasm.
    #[derive(Default)]
    pub struct WapcGuestData {
        index_slab0: wit_bindgen_wasmtime::rt::IndexSlab,
        resource_slab0: wit_bindgen_wasmtime::rt::ResourceSlab,
        dtor0: Option<wasmtime::TypedFunc<i32, ()>>,
    }
    pub struct WapcGuest<T> {
        get_state: Box<dyn Fn(&mut T) -> &mut WapcGuestData + Send + Sync>,
        canonical_abi_realloc: wasmtime::TypedFunc<(i32, i32, i32, i32), i32>,
        memory: wasmtime::Memory,
        wapc_init_guest_request: wasmtime::TypedFunc<(i32, i32, i32, i32, i32), (i32,)>,
        wapc_on_host_error: wasmtime::TypedFunc<(i32, i32, i32, i32), ()>,
        wapc_on_host_response: wasmtime::TypedFunc<(i32, i32, i32, i32, i32), ()>,
    }
    impl<T> WapcGuest<T> {
        /// Adds any intrinsics, if necessary for this exported wasm
        /// functionality to the `linker` provided.
        ///
        /// The `get_state` closure is required to access the
        /// auxiliary data necessary for these wasm exports from
        /// the general store's state.
        pub fn add_to_linker(
            linker: &mut wasmtime::Linker<T>,
            get_state: impl Fn(&mut T) -> &mut WapcGuestData + Send + Sync + Copy + 'static,
        ) -> anyhow::Result<()> {
            linker.func_wrap(
                "canonical_abi",
                "resource_drop_wapc",
                move |mut caller: wasmtime::Caller<'_, T>, idx: u32| {
                    let state = get_state(caller.data_mut());
                    let resource_idx = state.index_slab0.remove(idx)?;
                    let wasm = match state.resource_slab0.drop(resource_idx) {
                        Some(wasm) => wasm,
                        None => return Ok(()),
                    };
                    let dtor = state.dtor0.expect("destructor not set yet");
                    dtor.call(&mut caller, wasm)?;
                    Ok(())
                },
            )?;
            linker.func_wrap(
                "canonical_abi",
                "resource_clone_wapc",
                move |mut caller: wasmtime::Caller<'_, T>, idx: u32| {
                    let state = get_state(caller.data_mut());
                    let resource_idx = state.index_slab0.get(idx)?;
                    state.resource_slab0.clone(resource_idx)?;
                    Ok(state.index_slab0.insert(resource_idx))
                },
            )?;
            linker.func_wrap(
                "canonical_abi",
                "resource_get_wapc",
                move |mut caller: wasmtime::Caller<'_, T>, idx: u32| {
                    let state = get_state(caller.data_mut());
                    let resource_idx = state.index_slab0.get(idx)?;
                    Ok(state.resource_slab0.get(resource_idx))
                },
            )?;
            linker.func_wrap(
                "canonical_abi",
                "resource_new_wapc",
                move |mut caller: wasmtime::Caller<'_, T>, val: i32| {
                    let state = get_state(caller.data_mut());
                    let resource_idx = state.resource_slab0.insert(val);
                    Ok(state.index_slab0.insert(resource_idx))
                },
            )?;
            Ok(())
        }

        /// Instantiates the provided `module` using the specified
        /// parameters, wrapping up the result in a structure that
        /// translates between wasm and the host.
        ///
        /// The `linker` provided will have intrinsics added to it
        /// automatically, so it's not necessary to call
        /// `add_to_linker` beforehand. This function will
        /// instantiate the `module` otherwise using `linker`, and
        /// both an instance of this structure and the underlying
        /// `wasmtime::Instance` will be returned.
        ///
        /// The `get_state` parameter is used to access the
        /// auxiliary state necessary for these wasm exports from
        /// the general store state `T`.
        pub fn instantiate(
            mut store: impl wasmtime::AsContextMut<Data = T>,
            module: &wasmtime::Module,
            linker: &mut wasmtime::Linker<T>,
            get_state: impl Fn(&mut T) -> &mut WapcGuestData + Send + Sync + Copy + 'static,
        ) -> anyhow::Result<(Self, wasmtime::Instance)> {
            Self::add_to_linker(linker, get_state)?;
            let instance = linker.instantiate(&mut store, module)?;
            Ok((Self::new(store, &instance, get_state)?, instance))
        }

        /// Low-level creation wrapper for wrapping up the exports
        /// of the `instance` provided in this structure of wasm
        /// exports.
        ///
        /// This function will extract exports from the `instance`
        /// defined within `store` and wrap them all up in the
        /// returned structure which can be used to interact with
        /// the wasm module.
        pub fn new(
            mut store: impl wasmtime::AsContextMut<Data = T>,
            instance: &wasmtime::Instance,
            get_state: impl Fn(&mut T) -> &mut WapcGuestData + Send + Sync + Copy + 'static,
        ) -> anyhow::Result<Self> {
            let mut store = store.as_context_mut();
            let canonical_abi_realloc = instance.get_typed_func::<(i32, i32, i32, i32), i32, _>(
                &mut store,
                "canonical_abi_realloc",
            )?;
            let memory = instance
                .get_memory(&mut store, "memory")
                .ok_or_else(|| anyhow::anyhow!("`memory` export not a memory"))?;
            let wapc_init_guest_request = instance
                .get_typed_func::<(i32, i32, i32, i32, i32), (i32,), _>(
                    &mut store,
                    "wapc::init-guest-request",
                )?;
            let wapc_on_host_error = instance
                .get_typed_func::<(i32, i32, i32, i32), (), _>(&mut store, "wapc::on-host-error")?;
            let wapc_on_host_response = instance
                .get_typed_func::<(i32, i32, i32, i32, i32), (), _>(
                    &mut store,
                    "wapc::on-host-response",
                )?;

            get_state(store.data_mut()).dtor0 =
                Some(instance.get_typed_func::<i32, (), _>(&mut store, "canonical_abi_drop_wapc")?);

            Ok(WapcGuest {
                canonical_abi_realloc,
                memory,
                wapc_init_guest_request,
                wapc_on_host_error,
                wapc_on_host_response,
                get_state: Box::new(get_state),
            })
        }
        pub fn wapc_init_guest_request(
            &self,
            mut caller: impl wasmtime::AsContextMut<Data = T>,
            self_: &Wapc,
            operation: &str,
            payload: &[u8],
        ) -> Result<u32, wasmtime::Trap> {
            let func_canonical_abi_realloc = &self.canonical_abi_realloc;
            let memory = &self.memory;

            let obj0 = self_;
            (self.get_state)(caller.as_context_mut().data_mut())
                .resource_slab0
                .clone(obj0.0)?;
            let handle0 = (self.get_state)(caller.as_context_mut().data_mut())
                .index_slab0
                .insert(obj0.0);
            let vec1 = operation;
            let ptr1 =
                func_canonical_abi_realloc.call(&mut caller, (0, 0, 1, vec1.len() as i32))?;
            memory
                .data_mut(&mut caller)
                .store_many(ptr1, vec1.as_bytes())?;
            let vec2 = payload;
            let ptr2 =
                func_canonical_abi_realloc.call(&mut caller, (0, 0, 1, (vec2.len() as i32) * 1))?;
            memory.data_mut(&mut caller).store_many(ptr2, &vec2)?;
            let (result3_0,) = self.wapc_init_guest_request.call(
                &mut caller,
                (
                    handle0 as i32,
                    ptr1,
                    vec1.len() as i32,
                    ptr2,
                    vec2.len() as i32,
                ),
            )?;
            Ok(result3_0 as u32)
        }
        pub fn wapc_on_host_response(
            &self,
            mut caller: impl wasmtime::AsContextMut<Data = T>,
            self_: &Wapc,
            id: u32,
            code: u32,
            bytes: &[u8],
        ) -> Result<(), wasmtime::Trap> {
            let func_canonical_abi_realloc = &self.canonical_abi_realloc;
            let memory = &self.memory;

            let obj0 = self_;
            (self.get_state)(caller.as_context_mut().data_mut())
                .resource_slab0
                .clone(obj0.0)?;
            let handle0 = (self.get_state)(caller.as_context_mut().data_mut())
                .index_slab0
                .insert(obj0.0);
            let vec1 = bytes;
            let ptr1 =
                func_canonical_abi_realloc.call(&mut caller, (0, 0, 1, (vec1.len() as i32) * 1))?;
            memory.data_mut(&mut caller).store_many(ptr1, &vec1)?;
            self.wapc_on_host_response.call(
                &mut caller,
                (
                    handle0 as i32,
                    wit_bindgen_wasmtime::rt::as_i32(id),
                    wit_bindgen_wasmtime::rt::as_i32(code),
                    ptr1,
                    vec1.len() as i32,
                ),
            )?;
            Ok(())
        }
        pub fn wapc_on_host_error(
            &self,
            mut caller: impl wasmtime::AsContextMut<Data = T>,
            self_: &Wapc,
            id: u32,
            bytes: &[u8],
        ) -> Result<(), wasmtime::Trap> {
            let func_canonical_abi_realloc = &self.canonical_abi_realloc;
            let memory = &self.memory;

            let obj0 = self_;
            (self.get_state)(caller.as_context_mut().data_mut())
                .resource_slab0
                .clone(obj0.0)?;
            let handle0 = (self.get_state)(caller.as_context_mut().data_mut())
                .index_slab0
                .insert(obj0.0);
            let vec1 = bytes;
            let ptr1 =
                func_canonical_abi_realloc.call(&mut caller, (0, 0, 1, (vec1.len() as i32) * 1))?;
            memory.data_mut(&mut caller).store_many(ptr1, &vec1)?;
            self.wapc_on_host_error.call(
                &mut caller,
                (
                    handle0 as i32,
                    wit_bindgen_wasmtime::rt::as_i32(id),
                    ptr1,
                    vec1.len() as i32,
                ),
            )?;
            Ok(())
        }

        /// Drops the host-owned handle to the resource
        /// specified.
        ///
        /// Note that this may execute the WebAssembly-defined
        /// destructor for this type. This also may not run
        /// the destructor if there are still other references
        /// to this type.
        pub fn drop_wapc(
            &self,
            mut store: impl wasmtime::AsContextMut<Data = T>,
            val: Wapc,
        ) -> Result<(), wasmtime::Trap> {
            let mut store = store.as_context_mut();
            let data = (self.get_state)(store.data_mut());
            let wasm = match data.resource_slab0.drop(val.0) {
                Some(val) => val,
                None => return Ok(()),
            };
            data.dtor0.unwrap().call(&mut store, wasm)?;
            Ok(())
        }
    }
    use wit_bindgen_wasmtime::rt::RawMem;
}
#[allow(clippy::all)]
pub mod wapc_host {
    #[allow(unused_imports)]
    use wit_bindgen_wasmtime::{anyhow, wasmtime};
    pub trait WapcHost: Sized {
        type Wapc: std::fmt::Debug;
        fn wapc_init_host_request(
            &mut self,
            self_: &Self::Wapc,
            binding: &str,
            namespace: &str,
            operation: &str,
            bytes: &[u8],
        ) -> u32;

        fn wapc_on_guest_response(&mut self, self_: &Self::Wapc, id: u32, bytes: &[u8]) -> ();

        fn wapc_on_guest_error(&mut self, self_: &Self::Wapc, id: u32, bytes: &[u8]) -> ();

        fn wapc_console_log(&mut self, self_: &Self::Wapc, message: &str) -> ();

        fn drop_wapc(&mut self, state: Self::Wapc) {
            drop(state);
        }
    }

    pub struct WapcHostTables<T: WapcHost> {
        pub(crate) wapc_table: wit_bindgen_wasmtime::Table<T::Wapc>,
    }
    impl<T: WapcHost> Default for WapcHostTables<T> {
        fn default() -> Self {
            Self {
                wapc_table: Default::default(),
            }
        }
    }
    pub fn add_to_linker<T, U>(
        linker: &mut wasmtime::Linker<T>,
        get: impl Fn(&mut T) -> (&mut U, &mut WapcHostTables<U>) + Send + Sync + Copy + 'static,
    ) -> anyhow::Result<()>
    where
        U: WapcHost,
    {
        use wit_bindgen_wasmtime::rt::get_memory;
        linker.func_wrap(
            "wapc-host",
            "wapc::init-host-request",
            move |mut caller: wasmtime::Caller<'_, T>,
                  arg0: i32,
                  arg1: i32,
                  arg2: i32,
                  arg3: i32,
                  arg4: i32,
                  arg5: i32,
                  arg6: i32,
                  arg7: i32,
                  arg8: i32| {
                let memory = &get_memory(&mut caller, "memory")?;
                let (mem, data) = memory.data_and_store_mut(&mut caller);
                let mut _bc = wit_bindgen_wasmtime::BorrowChecker::new(mem);
                let host = get(data);
                let (host, _tables) = host;
                let ptr0 = arg1;
                let len0 = arg2;
                let ptr1 = arg3;
                let len1 = arg4;
                let ptr2 = arg5;
                let len2 = arg6;
                let ptr3 = arg7;
                let len3 = arg8;
                let param0 = _tables
                    .wapc_table
                    .get((arg0) as u32)
                    .ok_or_else(|| wasmtime::Trap::new("invalid handle index"))?;
                let param1 = _bc.slice_str(ptr0, len0)?;
                let param2 = _bc.slice_str(ptr1, len1)?;
                let param3 = _bc.slice_str(ptr2, len2)?;
                let param4 = _bc.slice(ptr3, len3)?;
                let result = host.wapc_init_host_request(param0, param1, param2, param3, param4);
                Ok(wit_bindgen_wasmtime::rt::as_i32(result))
            },
        )?;
        linker.func_wrap(
            "wapc-host",
            "wapc::on-guest-response",
            move |mut caller: wasmtime::Caller<'_, T>,
                  arg0: i32,
                  arg1: i32,
                  arg2: i32,
                  arg3: i32| {
                let memory = &get_memory(&mut caller, "memory")?;
                let (mem, data) = memory.data_and_store_mut(&mut caller);
                let mut _bc = wit_bindgen_wasmtime::BorrowChecker::new(mem);
                let host = get(data);
                let (host, _tables) = host;
                let ptr0 = arg2;
                let len0 = arg3;
                let param0 = _tables
                    .wapc_table
                    .get((arg0) as u32)
                    .ok_or_else(|| wasmtime::Trap::new("invalid handle index"))?;
                let param1 = arg1 as u32;
                let param2 = _bc.slice(ptr0, len0)?;
                let result = host.wapc_on_guest_response(param0, param1, param2);
                let () = result;
                Ok(())
            },
        )?;
        linker.func_wrap(
            "wapc-host",
            "wapc::on-guest-error",
            move |mut caller: wasmtime::Caller<'_, T>,
                  arg0: i32,
                  arg1: i32,
                  arg2: i32,
                  arg3: i32| {
                let memory = &get_memory(&mut caller, "memory")?;
                let (mem, data) = memory.data_and_store_mut(&mut caller);
                let mut _bc = wit_bindgen_wasmtime::BorrowChecker::new(mem);
                let host = get(data);
                let (host, _tables) = host;
                let ptr0 = arg2;
                let len0 = arg3;
                let param0 = _tables
                    .wapc_table
                    .get((arg0) as u32)
                    .ok_or_else(|| wasmtime::Trap::new("invalid handle index"))?;
                let param1 = arg1 as u32;
                let param2 = _bc.slice(ptr0, len0)?;
                let result = host.wapc_on_guest_error(param0, param1, param2);
                let () = result;
                Ok(())
            },
        )?;
        linker.func_wrap(
            "wapc-host",
            "wapc::console-log",
            move |mut caller: wasmtime::Caller<'_, T>, arg0: i32, arg1: i32, arg2: i32| {
                let memory = &get_memory(&mut caller, "memory")?;
                let (mem, data) = memory.data_and_store_mut(&mut caller);
                let mut _bc = wit_bindgen_wasmtime::BorrowChecker::new(mem);
                let host = get(data);
                let (host, _tables) = host;
                let ptr0 = arg1;
                let len0 = arg2;
                let param0 = _tables
                    .wapc_table
                    .get((arg0) as u32)
                    .ok_or_else(|| wasmtime::Trap::new("invalid handle index"))?;
                let param1 = _bc.slice_str(ptr0, len0)?;
                let result = host.wapc_console_log(param0, param1);
                let () = result;
                Ok(())
            },
        )?;
        linker.func_wrap(
            "canonical_abi",
            "resource_drop_wapc",
            move |mut caller: wasmtime::Caller<'_, T>, handle: u32| {
                let (host, tables) = get(caller.data_mut());
                let handle = tables
                    .wapc_table
                    .remove(handle)
                    .map_err(|e| wasmtime::Trap::new(format!("failed to remove handle: {}", e)))?;
                host.drop_wapc(handle);
                Ok(())
            },
        )?;
        Ok(())
    }
    use wit_bindgen_wasmtime::rt::RawMem;
}
