# Bouncing DVD logo

This Jack project features the famous Dvd logo bouncing around the screen.

The logo was downsized to 96x42 pixels & a Python script was used to encode it 16 bits at a time for the Hack computer screen.

The main challenge of this project was to move the sprite in the X dimension by anything other than 16 pixels at a time. The reason is that it's difficult to encode binary data in Jack & the image is difficult to move sideways without efficient bit-shifting. The solution was to render the image into 8 frames, moving it by 2 pixels in the X direction each time. The program then cycles through each of these frames.

https://github.com/guydunton/nandtotetris-tools/assets/14931831/096c85f8-5055-4bf4-9f44-8ab7cdee8ddb

