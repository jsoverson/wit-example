#[allow(clippy::all)]
mod wapc_host {
  #[derive(Debug)]
  #[repr(transparent)]
  pub struct Wapc(i32);
  impl Wapc{
    pub unsafe fn from_raw(raw: i32) -> Self {
      Self(raw)
    }
    
    pub fn into_raw(self) -> i32 {
      let ret = self.0;
      core::mem::forget(self);
      return ret;
    }
    
    pub fn as_raw(&self) -> i32 {
      self.0
    }
  }
  impl Drop for Wapc{
    fn drop(&mut self) {
      #[link(wasm_import_module = "canonical_abi")]
      extern "C" {
        #[link_name = "resource_drop_wapc"]
        fn close(fd: i32);
      }
      unsafe {
        close(self.0);
      }
    }
  }
  impl Clone for Wapc{
    fn clone(&self) -> Self {
      #[link(wasm_import_module = "canonical_abi")]
      extern "C" {
        #[link_name = "resource_clone_wapc"]
        fn clone(val: i32) -> i32;
      }
      unsafe {
        Self(clone(self.0))
      }
    }
  }
  impl Wapc {
    pub fn init_host_request(&self,binding: & str,namespace: & str,operation: & str,bytes: &[u8],) -> u32{
      unsafe {
        let vec0 = binding;
        let ptr0 = vec0.as_ptr() as i32;
        let len0 = vec0.len() as i32;
        let vec1 = namespace;
        let ptr1 = vec1.as_ptr() as i32;
        let len1 = vec1.len() as i32;
        let vec2 = operation;
        let ptr2 = vec2.as_ptr() as i32;
        let len2 = vec2.len() as i32;
        let vec3 = bytes;
        let ptr3 = vec3.as_ptr() as i32;
        let len3 = vec3.len() as i32;
        #[link(wasm_import_module = "wapc-host")]
        extern "C" {
          #[cfg_attr(target_arch = "wasm32", link_name = "wapc::init-host-request")]
          #[cfg_attr(not(target_arch = "wasm32"), link_name = "wapc-host_wapc::init-host-request")]
          fn wit_import(_: i32, _: i32, _: i32, _: i32, _: i32, _: i32, _: i32, _: i32, _: i32, ) -> i32;
        }
        let ret = wit_import(self.0, ptr0, len0, ptr1, len1, ptr2, len2, ptr3, len3);
        ret as u32
      }
    }
  }
  impl Wapc {
    pub fn on_guest_response(&self,id: u32,bytes: &[u8],) -> (){
      unsafe {
        let vec0 = bytes;
        let ptr0 = vec0.as_ptr() as i32;
        let len0 = vec0.len() as i32;
        #[link(wasm_import_module = "wapc-host")]
        extern "C" {
          #[cfg_attr(target_arch = "wasm32", link_name = "wapc::on-guest-response")]
          #[cfg_attr(not(target_arch = "wasm32"), link_name = "wapc-host_wapc::on-guest-response")]
          fn wit_import(_: i32, _: i32, _: i32, _: i32, );
        }
        wit_import(self.0, wit_bindgen_rust::rt::as_i32(id), ptr0, len0);
        ()
      }
    }
  }
  impl Wapc {
    pub fn on_guest_error(&self,id: u32,bytes: &[u8],) -> (){
      unsafe {
        let vec0 = bytes;
        let ptr0 = vec0.as_ptr() as i32;
        let len0 = vec0.len() as i32;
        #[link(wasm_import_module = "wapc-host")]
        extern "C" {
          #[cfg_attr(target_arch = "wasm32", link_name = "wapc::on-guest-error")]
          #[cfg_attr(not(target_arch = "wasm32"), link_name = "wapc-host_wapc::on-guest-error")]
          fn wit_import(_: i32, _: i32, _: i32, _: i32, );
        }
        wit_import(self.0, wit_bindgen_rust::rt::as_i32(id), ptr0, len0);
        ()
      }
    }
  }
  impl Wapc {
    pub fn console_log(&self,message: & str,) -> (){
      unsafe {
        let vec0 = message;
        let ptr0 = vec0.as_ptr() as i32;
        let len0 = vec0.len() as i32;
        #[link(wasm_import_module = "wapc-host")]
        extern "C" {
          #[cfg_attr(target_arch = "wasm32", link_name = "wapc::console-log")]
          #[cfg_attr(not(target_arch = "wasm32"), link_name = "wapc-host_wapc::console-log")]
          fn wit_import(_: i32, _: i32, _: i32, );
        }
        wit_import(self.0, ptr0, len0);
        ()
      }
    }
  }
}
#[allow(clippy::all)]
mod wapc_guest {
  
  unsafe impl wit_bindgen_rust::HandleType for super::Wapc {
    #[inline]
    fn clone(_val: i32) -> i32 {
      
      #[cfg(not(target_arch = "wasm32"))]
      {
        panic!("handles can only be used on wasm32");
      }
      #[cfg(target_arch = "wasm32")]
      
      {
        #[link(wasm_import_module = "canonical_abi")]
        extern "C" {
          #[link_name = "resource_clone_wapc"]
          fn clone(val: i32) -> i32;
        }
        unsafe { clone(_val) }
      }
    }
    
    #[inline]
    fn drop(_val: i32) {
      
      #[cfg(not(target_arch = "wasm32"))]
      {
        panic!("handles can only be used on wasm32");
      }
      #[cfg(target_arch = "wasm32")]
      
      {
        #[link(wasm_import_module = "canonical_abi")]
        extern "C" {
          #[link_name = "resource_drop_wapc"]
          fn drop(val: i32);
        }
        unsafe { drop(_val) }
      }
    }
  }
  
  unsafe impl wit_bindgen_rust::LocalHandle for super::Wapc {
    #[inline]
    fn new(_val: i32) -> i32 {
      
      #[cfg(not(target_arch = "wasm32"))]
      {
        panic!("handles can only be used on wasm32");
      }
      #[cfg(target_arch = "wasm32")]
      
      {
        #[link(wasm_import_module = "canonical_abi")]
        extern "C" {
          #[link_name = "resource_new_wapc"]
          fn new(val: i32) -> i32;
        }
        unsafe { new(_val) }
      }
    }
    
    #[inline]
    fn get(_val: i32) -> i32 {
      
      #[cfg(not(target_arch = "wasm32"))]
      {
        panic!("handles can only be used on wasm32");
      }
      #[cfg(target_arch = "wasm32")]
      
      {
        #[link(wasm_import_module = "canonical_abi")]
        extern "C" {
          #[link_name = "resource_get_wapc"]
          fn get(val: i32) -> i32;
        }
        unsafe { get(_val) }
      }
    }
  }
  
  const _: () = {
    #[export_name = "canonical_abi_drop_wapc"]
    extern "C" fn drop(ty: Box<super::Wapc>) {
      <super::WapcGuest as WapcGuest>::drop_wapc(*ty)
    }
  };
  #[export_name = "wapc::init-guest-request"]
  unsafe extern "C" fn __wit_bindgen_wapc_guest_wapc_init_guest_request(arg0: i32, arg1: i32, arg2: i32, arg3: i32, arg4: i32, ) -> i32{
    let len0 = arg2 as usize;
    let len1 = arg4 as usize;
    let result = <super::Wapc as Wapc>::init_guest_request(&wit_bindgen_rust::Handle::from_raw(arg0), String::from_utf8(Vec::from_raw_parts(arg1 as *mut _, len0, len0)).unwrap(), Vec::from_raw_parts(arg3 as *mut _, len1, len1));
    wit_bindgen_rust::rt::as_i32(result)
  }
  #[export_name = "wapc::on-host-response"]
  unsafe extern "C" fn __wit_bindgen_wapc_guest_wapc_on_host_response(arg0: i32, arg1: i32, arg2: i32, arg3: i32, arg4: i32, ){
    let len0 = arg4 as usize;
    let result = <super::Wapc as Wapc>::on_host_response(&wit_bindgen_rust::Handle::from_raw(arg0), arg1 as u32, arg2 as u32, Vec::from_raw_parts(arg3 as *mut _, len0, len0));
    let () = result;
  }
  #[export_name = "wapc::on-host-error"]
  unsafe extern "C" fn __wit_bindgen_wapc_guest_wapc_on_host_error(arg0: i32, arg1: i32, arg2: i32, arg3: i32, ){
    let len0 = arg3 as usize;
    let result = <super::Wapc as Wapc>::on_host_error(&wit_bindgen_rust::Handle::from_raw(arg0), arg1 as u32, Vec::from_raw_parts(arg2 as *mut _, len0, len0));
    let () = result;
  }
  pub trait WapcGuest {
    
    /// An optional callback invoked when a handle is finalized
    /// and destroyed.
    fn drop_wapc(val: super::Wapc) {
      drop(val);
    }
    
  }
  pub trait Wapc {
    fn init_guest_request(&self,operation: String,payload: Vec<u8>,) -> u32;
    fn on_host_response(&self,id: u32,code: u32,bytes: Vec<u8>,) -> ();
    fn on_host_error(&self,id: u32,bytes: Vec<u8>,) -> ();
  }
}
