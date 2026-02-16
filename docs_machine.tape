# Run: vhs docs_machine.tape
Output apps/landing/public/assets/docs_machine.webm
Set FontSize 14
Set Width 800
Set Height 500
Set Padding 20
Set Theme "Catppuccin Mocha"

# Setup
Hide
Type "rm -rf .chisel"
Enter
Type "chisel init"
Enter
Sleep 1s
# Clear defaults
Type "rm -rf .chisel/docs/*"
Enter
# Create directory
Type "mkdir -p .chisel/docs/incidents"
Enter
# Copy demo doc
Type "cp demo_data/doc_1.md .chisel/docs/incidents/postmortem.md"
Enter
Sleep 1s
Type "clear"
Enter
Show

Type "chisel docs show incidents/postmortem.md --machine"
Sleep 500ms
Enter
Sleep 4s
