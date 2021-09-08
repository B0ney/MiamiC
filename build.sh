#cargo build --release -Zmultitarget --target=x86_64-unknown-linux-gnu --target=x86_64-pc-windows-gnu 

cargo +nightly build -Z build-std=std,panic_abort -Zmultitarget --target=x86_64-unknown-linux-gnu --target=x86_64-pc-windows-gnu --target=i686-pc-windows-gnu --release
#cargo +nightly build -Z build-std=std,panic_abort --target=i686-unknown-linux-gnu

cp ./target/x86_64-unknown-linux-gnu/release/MiamiC ./build/linux/miamic
cp ./target/x86_64-pc-windows-gnu/release/MiamiC.exe ./build/windows/miamic.exe
#cp ./target/i686-pc-windows-gnu/release/MiamiC.exe ./build/windows/miamic_32.exe
