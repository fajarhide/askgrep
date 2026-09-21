# Regenerate with:  TYPESAFE_API_KEY=... vhs demo.tape
# The key is read from the environment and never typed, so it cannot reach the GIF.
# The hidden setup clears the answer cache so the first sweep shows its real cost.

Output media/demo.gif

Set Shell bash
Set FontSize 18
Set Width 1240
Set Height 330
Set Padding 24
Set Theme "Catppuccin Mocha"
Set TypingSpeed 35ms

Hide
Type "export PATH=$PWD/target/release:$PATH PS1='$ '"
Enter
Type "rm -rf ~/.cache/askgrep && clear"
Enter
Sleep 1s
Show

Type "askgrep 'builds an SQL query by concatenating a value that came from the request.' demo/"
Sleep 600ms
Enter
Sleep 7s

Type "askgrep 'builds an SQL query by concatenating a value that came from the request.' demo/"
Sleep 400ms
Enter
Sleep 3s
Type "# same answer, cached, free"
Sleep 1500ms
Enter
Sleep 2500ms
