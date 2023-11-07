# File Format

Code is encoded as UTF-8 text format. UTF-8 encoded files can be loaded without any additional encoding step.

All built-in, core and standard-library code is compatible with ASCII encoding, which makes it easier to convert ASCII encoded files to UTF-8 as a pre-processing step, in case this is necessary on some platforms.

Files may have both `\n` and `\r\n` line endings. The carriage return `\r` is ignored internally.

Files should use the `.dragon` extension, `.drg` may alternatively be used if and only if the file system requires extensions to be at most 3 characters long.
