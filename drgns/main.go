package main

import (
	"fmt"

	"github.com/juju/gnuflag"
)

func main() {
	inputPtr := gnuflag.String("input", "", "run in batch mode, given the path to the entry-point file")
	checkPtr := gnuflag.Bool("check", false, "check program without running it")
	gnuflag.Parse(true)

	// post-process flags, because gnuflag is very simple (which I like)
	use_repl_mode := *inputPtr == ""
	input := *inputPtr
	check := *checkPtr

	if use_repl_mode {

	} else {

	}

	fmt.Println(input)
	fmt.Println(use_repl_mode)
	fmt.Println(check)

}
