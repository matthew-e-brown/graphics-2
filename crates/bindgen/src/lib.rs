use std::io::Write;


pub fn write_bindings<T: Write>(mut output: T) -> std::io::Result<()> {
    output.write("// Hello, World!\n".as_bytes()).map(|_| ())
}
