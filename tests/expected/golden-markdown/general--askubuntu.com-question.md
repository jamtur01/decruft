What I'm trying to do is increase the font size, not change the resolution. I still want my resolution to be 4k, but I want a bigger font size. Is this possible? This is the terminal, there is no GUI installed.

Edit: the answer was: sudo vi /etc/default/console-setup FONTFACE="Terminus" FONTSIZE="16x32" sudo update-initramfs -u sudo reboot

It would be nice also to change the console resolution but I don't know how in Ubuntu Server 20.04 LTS

* * *

## Comments

> **Octavian-Codrut Popescu** · · 19 votes
> 
> The answer was to edit the file `/etc/default/console-setup` and enter:
> 
> ```
> FONTFACE="Terminus"
> FONTSIZE="16x32"
> ```
> 
> then:
> 
> ```
> sudo update-initramfs -u
> sudo reboot
> ```

> **Manu Mathur** · · 3 votes
> 
> Do help us out with the name of which terminal application you're using.
> 
> ***Part I: Increasing the Font size of your Terminal window (if its GNOME):***
> 
> Here's how you can do it:
> 
> *   Open your terminal window \[Shortcut: `Ctrl` + `Alt` + `T`\]
> 
> *   Click the hamburger option in the top right corner of the terminal window >> Click on Preferences option OR
> 
> *   Right-Click anywhere within the terminal window >> Choose Preferences option from the context sensitive menu
> 
> *   On the preferences window, go to "Text" tab
> 
> *   Click “custom font” check box to enable the font customization and alter the Font style and Font size
> 
> *   Click on "Font-size" input field >> Pick your font style and move the slider (at the bottom) to determine your ideal font size
> 
> *   Click on "Select" to ensure your chosen font style & size get implemented.
> 
> *   Close the preferences window and save your settings.
> 
> Your terminal should now showcase the new Font style and size.
> 
> ***Part II: If you're talking about the generic font size***
> 
> *   Install Gnome tool using the command: `sudo apt install gnome-tweak-tool`
> 
> *   Open Gnome tool >> Click "Fonts" tab
> 
> *   Configure font settings for 'Window Title', 'Interface', 'Document', and 'Monospace' through this tab.
> 
> *   Increase/Decrease the "Scaling Factor" to change the general Font size for your Ubuntu Desktop.
> 
> *   If required, remove Gnome by entering the command: `$ sudo apt remove gnome-tweak-tool`
> 
> Let us know how it works out for you.
> 
> Cheers,
> 
> Manu

> **ealbrechtsen** · · 0 votes
> 
> Using 16x32 as suggested does fix the issue of the font being too small. However, in my case (MacBook Pro 2017 running Ubuntu Server), it introduced another problem.
> 
> The larger font reduced the number of visible rows so much that command output extended past the bottom of the screen. This made it impossible to see the final lines or know when a command had finished.
> 
> Switching to a smaller size like 16x16 or 12x24 provided a better balance between readability and usable screen space.