## Format total commander list output
### Command
```
cargo run -- --list --path %WL
```
### Input
```
d:\Dir\title\key｜title｜subtitle1.zip
d:\Dir\title\key｜title｜subtitle8.zip
d:\Dir\title\key｜title｜subtitle3.zip
d:\Dir\title\key｜title｜subtitle5.zip
d:\Dir\title\key｜title｜subtitle9.zip
d:\Dir\title\key  title subtitle999.zip
d:\Dir\title2\key2｜title2｜subtitlex.zip
```
### Output
```
title

key2
  subtitlex

key
  subtitle1
  subtitle3
  subtitle5
  subtitle8
  subtitle9

<>
  subtitle999
```
