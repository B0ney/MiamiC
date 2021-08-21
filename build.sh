#cargo build --release -Zmultitarget --target=x86_64-unknown-linux-gnu --target=x86_64-pc-windows-gnu 

cargo +nightly build -Z build-std=std,panic_abort -Zmultitarget --target=x86_64-unknown-linux-gnu --target=x86_64-pc-windows-gnu --release

cp ./target/x86_64-unknown-linux-gnu/release/ViceConv ./build/linux/miamic
cp ./target/x86_64-pc-windows-gnu/release/ViceConv.exe ./build/windows/miamic.exe

