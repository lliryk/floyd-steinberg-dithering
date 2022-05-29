# Floyd-Steinberg-Dithering

An implemenation of the floyd-steinberg dithering algorithim in Rust.

Based on [javidx9's implementation](https://www.youtube.com/watch?v=lseR6ZguBNY) in C++ ([Source](https://github.com/OneLoneCoder/olcPixelGameEngine/blob/master/Videos/OneLoneCoder_PGE_Dithering.cpp))

The implementation only works on the most basic BitMap (.bmp) images and implements a naive way of quantizing colors. 

It's my first time using Rust so the code is rather naive and may not follow the best or recommended practices. 