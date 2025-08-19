build:
     cargo build --release --target armv7-unknown-linux-musleabihf -p runner

upload: build
    ssh pi@windowpi.hacklab -- sudo systemctl stop led-matrix
    ssh pi@windowpi.hacklab -- sudo pkill -f led-text-display
    scp target/armv7-unknown-linux-musleabihf/release/runner pi@windowpi.hacklab:.local/bin/led-text-display

run: upload
    ssh pi@windowpi.hacklab -- sudo RUST_LOG=debug /home/pi/.local/bin/led-text-display

deploy: upload
    ssh pi@windowpi.hacklab -- sudo systemctl restart led-matrix
