#!/bin/bash

# Turn on lights
# python3 /path/to/turn_on_lights.py

echo 'Hello and welcome aboard the Oh One Light. Buckle your seat belt and get ready for liftoff..' | /home/zachwe/code/piper/piper --model /home/zachwe/code/en_GB-northern_english_male-medium.onnx --output-raw |   aplay -r 22050 -f S16_LE -t raw -
echo 'Here is an inspirational quote from JFK to get started.' | /home/zachwe/code/piper/piper --model /home/zachwe/code/en_GB-northern_english_male-medium.onnx --output-raw |   aplay -r 22050 -f S16_LE -t raw -
# Play startup sound
aplay /home/zachwe/code/whisper.cpp/samples/jfk.wav

sleep 100

# Run your long-running Python script
# python3 /path/to/main_app.py

