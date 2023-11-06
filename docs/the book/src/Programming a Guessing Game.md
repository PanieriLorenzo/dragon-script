# Programming a Guessing Game

```python
import stl::random::rand_range

println("Guess the number!")

secret_number := rand_range(0..=100)

loop {
    println("Please input your guess.")

    mut guess: string | int | none := input()
    guess = parse<int>(guess)

    if guess is none {
        println("Your input needs to be an integer")
        continue
    }

    guess = guess.unwrap()

    if guess < secret_number {
        println("Too small!")
        continue
    }

    if guess > secret_number {
        println("Too big!")
        continue
    }

    println("You win!")
    break
}
```
