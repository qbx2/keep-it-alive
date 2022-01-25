use libc::*;
use std::mem::*;

#[ctor::ctor]
static CONNECT: unsafe extern "C" fn(
    sockfd: i32,
    addr: *const sockaddr,
    addrlen: socklen_t,
) -> i32 = {
    let ptr = dlsym(RTLD_NEXT, "connect\0".as_ptr() as *const c_char);
    if ptr.is_null() {
        panic!("no connect");
    }
    transmute(ptr)
};

unsafe fn set_socket_options(sockfd: i32) {
    let optval: i32 = 1;
    let ret = setsockopt(
        sockfd,
        SOL_SOCKET,
        SO_KEEPALIVE,
        &optval as *const i32 as *const c_void,
        size_of_val(&optval) as socklen_t,
    );

    if ret != 0 {
        eprintln!("Failed to set SO_KEEPALIVE={optval} on {sockfd}: ret={ret}");
        return;
    }

    let optval: i32 = 60;
    let ret = setsockopt(
        sockfd,
        SOL_TCP,
        TCP_KEEPIDLE,
        &optval as *const i32 as *const c_void,
        size_of_val(&optval) as socklen_t,
    );

    if ret != 0 {
        eprintln!("Failed to set TCP_KEEPIDLE={optval} on {sockfd}: ret={ret}");
        return;
    }

    let optval: i32 = 60;
    let ret = setsockopt(
        sockfd,
        SOL_TCP,
        TCP_KEEPINTVL,
        &optval as *const i32 as *const c_void,
        size_of_val(&optval) as socklen_t,
    );

    if ret != 0 {
        eprintln!("Failed to set TCP_KEEPINTVL={optval} on {sockfd}: ret={ret}");
        return;
    }

    let optval: i32 = 5;
    let ret = setsockopt(
        sockfd,
        SOL_TCP,
        TCP_KEEPCNT,
        &optval as *const i32 as *const c_void,
        size_of_val(&optval) as socklen_t,
    );

    if ret != 0 {
        eprintln!("Failed to set TCP_KEEPCNT={optval} on {sockfd}: ret={ret}");
        return;
    }
}

#[no_mangle]
pub unsafe extern "C" fn connect(sockfd: i32, addr: *const sockaddr, addrlen: socklen_t) -> i32 {
    if addrlen as usize == size_of::<sockaddr_in>() {
        let addr: &sockaddr_in = transmute(addr);

        if addr.sin_family == AF_INET as u16 {
            let mut optval: i32 = 0;
            let ret = getsockopt(
                sockfd,
                SOL_SOCKET,
                SO_TYPE,
                &mut optval as *mut i32 as *mut c_void,
                &mut (size_of_val(&optval) as socklen_t) as *mut socklen_t,
            );

            if ret == 0 {
                if optval == SOCK_STREAM {
                    set_socket_options(sockfd);
                }
            } else {
                eprintln!("Failed to get SO_TYPE on {sockfd}: ret={ret}");
            }
        }
    }

    let ret = CONNECT(sockfd, addr, addrlen);
    eprintln!("connect({sockfd}, {addr:?}, {addrlen}) -> {ret}");
    ret
}
