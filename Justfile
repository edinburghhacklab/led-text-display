build:
     cargo build --target armv7-unknown-linux-musleabihf

upload: build
    ssh pi@windowpi.hacklab -- sudo pkill -f led-text-display
    scp target/armv7-unknown-linux-musleabihf/debug/led-text-display pi@windowpi.hacklab:.local/bin/led-text-display

run: upload
    ssh pi@windowpi.hacklab -- sudo /home/pi/.local/bin/led-text-display
