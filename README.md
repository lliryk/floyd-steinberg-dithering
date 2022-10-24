# Floyd-Steinberg-Dithering

## Before
![Before](images/before.jpg)

## After (1-bit Red Black White)
![After](images/edited.jpg)
Photo by [Luke Richardson](https://unsplash.com/@lukealrich?utm_source=unsplash&utm_medium=referral&utm_content=creditCopyText") on [Unsplash](https://unsplash.com/s/photos/mountains?utm_source=unsplash&utm_medium=referral&utm_content=creditCopyText)
  
## Contents
An implemenation of the floyd-steinberg dithering algorithim in Rust.

Based on [javidx9's implementation](https://www.youtube.com/watch?v=lseR6ZguBNY) in C++ ([Source](https://github.com/OneLoneCoder/olcPixelGameEngine/blob/master/Videos/OneLoneCoder_PGE_Dithering.cpp))

The implementation only works on the most basic BitMap (.bmp) images and combineds to different methods of quantization which may cause glitches and artifacts.

My first attempt at Rust, so some things are slightly broken (i.e. order of colors effects quantization)