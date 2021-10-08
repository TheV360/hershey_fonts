# Hershey fonts

Toy implementation of [the Hershey font format](https://github.com/kamalmostafa/hershey-fonts/blob/master/hershey-fonts.notes) in Rust. The reader and viewer components are separate, so feel free to pull out the reader for your own stuff. I'm not 100% confident with my parser writing skills, but it parsed all of the default fonts without erroring, so...

Also, check out [hershey.txt](./hershey.txt) to see some history and the usage terms of the default fonts I've copied over here. Thank you!

## To do
* Viewer: char map should be nicer
* Viewer: mouse interaction
* Reader: maybe some better tests
