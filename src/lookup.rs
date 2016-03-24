use std::mem;
use std::ffi::{CString, NulError};
use std::ptr;
use std::io;
use std::net::SocketAddr;
use addr::{MySocketAddrV4, MySocketAddrV6};
use libc as c;

fn sockaddr_to_addr(storage: &c::sockaddr_storage,
          len: usize) -> io::Result<SocketAddr> {
  match storage.ss_family as c::c_int {
    c::AF_INET => {
      assert!(len as usize >= mem::size_of::<c::sockaddr_in>());
      Ok(
        MySocketAddrV4
          ::from(unsafe { *(storage as *const _ as *const c::sockaddr_in) })
          .into()
      )
    }
    c::AF_INET6 => {
      assert!(len as usize >= mem::size_of::<c::sockaddr_in6>());
      Ok(
        MySocketAddrV6
          ::from(unsafe { *(storage as *const _ as *const c::sockaddr_in6) })
          .into()
      )
    }
    _ => {
      Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid argument"))
    }
  }
}

#[derive(Debug)]
pub struct LookupHost {
  original: *mut c::addrinfo,
  cur: *mut c::addrinfo,
}

impl Iterator for LookupHost {
  type Item = io::Result<SocketAddr>;
  fn next(&mut self) -> Option<io::Result<SocketAddr>> {
  unsafe {
    if self.cur.is_null() { return None }
    let ret = sockaddr_to_addr(mem::transmute((*self.cur).ai_addr),
           (*self.cur).ai_addrlen as usize);
    self.cur = (*self.cur).ai_next as *mut c::addrinfo;
    Some(ret)
  }
  }
}

unsafe impl Sync for LookupHost {}
unsafe impl Send for LookupHost {}

impl Drop for LookupHost {
  fn drop(&mut self) {
  unsafe { c::freeaddrinfo(self.original) }
  }
}

pub fn lookup_host(host: &str) -> Result<LookupHost, self::Error> {
  // FIXME: THis should be called for Windows.
  //init();

  let c_host = try!(CString::new(host));
  let mut res = ptr::null_mut();
  unsafe {
    match c::getaddrinfo(c_host.as_ptr(), ptr::null(), ptr::null(), &mut res) {
      0 => Ok(LookupHost { original: res, cur: res }),
      _ => Err(Error::Generic),
    }
  }
}

// FIXME: To go from SocketAddr -> c socket ptr is a wee bit harder.
//
// pub fn lookup_addr(addr: &IpAddr) -> Result<LookupHost, self::Error> {
//   // FIXME: This should be called for Windows.
//   // init();
// 
//   let saddr = SocketAddr::new(*addr, 0);
//   let (inner, len) = saddr.into_inner();
//   let mut hostbuf = [0 as c_char; c::NI_MAXHOST as usize];
// 
//   let data = unsafe {
//     try!(cvt_gai(c::getnameinfo(inner, len,
//                   hostbuf.as_mut_ptr(),
//                   c::NI_MAXHOST,
//                   ptr::null_mut(), 0, 0)));
// 
//     CStr::from_ptr(hostbuf.as_ptr())
//   };
// 
//   match from_utf8(data.to_bytes()) {
//     Ok(name) => Ok(name.to_owned()),
//     Err(_) => Err(io::Error::new(io::ErrorKind::Other,
//                    "failed to lookup address information"))
//   }
// }

#[derive(Debug)]
pub enum Error {
  IOError(io::Error),
  NulError(NulError),
  Generic
}

impl From<io::Error> for Error {
  fn from(err: io::Error) -> Self {
    Error::IOError(err)
  }
}

impl From<NulError> for Error {
  fn from(err: NulError) -> Self {
    Error::NulError(err)
  }
}

#[test]
fn test_localhost() {
  // FIXME: I should test the values I get back
  let _ = lookup_host("localhost").unwrap().collect::<Result<Vec<_>, _>>().unwrap();
}
