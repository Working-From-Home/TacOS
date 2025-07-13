use core::arch::asm;

pub fn inb(port: u16) -> u8 {
    let mut ret: u8;
    unsafe {
        asm!(
            "in al, dx",    // lit un byte du port dans dx vers al
            out("al") ret,  // la sortie est dans al -> ret
            in("dx") port,  // l'entrée est dans dx -> port
        );
    }
    ret
}

pub fn outb(port: u16, val: u8) {
    unsafe {
        asm!(
            "out dx, al",    // envoie un byte de al vers port dans dx
            in("dx") port,
            in("al") val,
        );
    }
}