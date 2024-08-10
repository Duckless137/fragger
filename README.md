![1723243069734](images/README/logo.png)

# About

**Fragger** is a lightweight application used to split large files up in order to be sent through messaging apps such as Discord or even regular SMS messaging. Friends who have the application will be able to reassemble the files.

# Limitations

* File chunks can only be 1KB small
* File chunks can only be 100MB large
* Files can only be 4TB large

# How to use

## Splitting

1. Select a fragment file size from the preset tabs above, or use the "custom" tab. The custom tab allows you to enter any file size in KB. If you enter a size with MB at the end, the size will be read as MB and automatically converteed into KB.
2. After selecting a fragment size, press the "Split" button and select a file to split. Fragger will create a directory with the fragment files in the directory of the original file.
3. Send the fragment (.frag) files one at time.

## Reassembling

1. After receiving the fragment (.frag) files from your friend, place them in a new directory.
2. Open the program and press the "Reassemble" button. Select the directory with the .frag files.
3. The reassbled file will be placed in the parent path of the fragment directory.\

# Contribution

Frankly, my code is so messy and bad ANY contribution would be much appreciated. Pull requests are welcome, but please open an issue for large changes, and discuss with me first.
Here are a few things planned for this project, for which help would be appreciated:
* Stopping Windows from immediatley identifying it as malware
* Optimization
* Dynamic file id bytes
* Linux support
* MacOS support

