Output ../demo.gif
Set FontSize 14
Set Width 1200
Set Height 700
Set WindowBar Colorful
Set Margin 20
Set MarginFill "#c566ff"
Set BorderRadius 10
Set LineHeight 1.3

Type "tree"
Sleep 0.5s
Enter
Sleep 2.5s

Type "cat contacts/alice.firm"
Sleep 0.5s
Enter
Sleep 3s

Type "firm related contact alice_acme"
Sleep 0.5s
Enter
Sleep 3s

Type `firm add --type task --id review --field name 'Review' --field project_ref 'project.website'`
Sleep 0.5s
Enter
Sleep 2.5s

Type `firm query 'from project | where status == "active" | related task'`
Sleep 0.5s
Enter
Sleep 4s
