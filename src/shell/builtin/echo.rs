use crate::printkln;

pub fn echo(args: &[u8]) {
    if args.is_empty() {
        printkln!();
    } else {
        printkln!("{}", args);
    }
}
