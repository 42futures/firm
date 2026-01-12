Output ../media/demo.gif
Set FontSize 14
Set Width 1000
Set Height 600
Set WindowBar Colorful
Set Margin 20
Set MarginFill "#c566ff"
Set BorderRadius 10
Set LineHeight 1.3

Type "tree"
Sleep 0.5s
Enter
Sleep 2s

Type "cat core/main.firm"
Sleep 0.5s
Enter
Sleep 3s

Type "clear"
Sleep 0.5s
Enter
Sleep 500ms

# List entities
Type "firm build"
Sleep 0.5s
Enter
Sleep 2s

# List entities
Type "firm -c list contact"
Sleep 0.5s
Enter
Sleep 3s

# The killer feature: traverse the graph
Type "firm -c related contact kent_smith"
Sleep 0.5s
Enter
Sleep 3s

Type "firm -c get task kent_lunch_meeting"
Sleep 0.5s
Enter
Sleep 3s
