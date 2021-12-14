#! /bin/sh

convert -delay 10 -loop 0 ./rendered/*.png animation.gif
gifsicle -i animation.gif -O3 --colors 256 -o anim-opt.gif