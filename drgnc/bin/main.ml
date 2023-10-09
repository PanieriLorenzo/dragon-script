open Lib;;

let maybe_read_line () =
  try Some(read_line())
with End_of_file -> None

let rec repl () =
  match maybe_read_line() with
  | Some(_) -> (repl [@tailcall]) ()
  | None -> ();;

repl ()
