### Highlights

*   [Here’s our Firefox Year in Review!](https://blog.mozilla.org/blog/2020/12/15/our-year-in-review-how-weve-kept-firefox-working-for-you-in-2020/)
*   [Here’s our Performance Year in Review!](https://blog.mozilla.org/performance/2020/12/15/2020-year-in-review/)
*   We’ve just landed [Bug 1553982](https://bugzilla.mozilla.org/show_bug.cgi?id=1553982), which aims to prevent starting an update while another Firefox instance is running (the cause of that about:restartrequired error page you may have seen).
    *   [![The about:restartrequired error page, saying \"Sorry. We just need to do one small thing to keep going. Nightly has just been updated in the background. Click Restart Nightly to complete the update. We will restore all your pages, windows and tabs afterwards, so you can be on your way quickly.\", followed by a button to restart Nightly.](https://3sgkpvh31s44756j71xlti9b-wpengine.netdna-ssl.com/files/2020/12/headlines85_0.png)](https://3sgkpvh31s44756j71xlti9b-wpengine.netdna-ssl.com/files/2020/12/headlines85_0.png)

        Users who run multiple user profiles concurrently will probably see this less!
*   Also just about to land is [Bug 353804](https://bugzilla.mozilla.org/show_bug.cgi?id=353804), which provides some support for downloading new updates when we already have an update downloaded but haven’t installed it yet. That should prevent many cases of restarting to finish an update and then immediately being notified about another one.
*   Thanks to evilpie, users can now [import logins from Keepass(XC) into Firefox](https://bugzilla.mozilla.org/show_bug.cgi?id=1650645)
*   From Firefox 85 it’s now possible to disable tab-to-search on a per-engine basis, by unchecking a search engine in *Search Preferences*. That will both hide the shortcut button and disable tab-to-search for the engine. ([Bug 1681512](https://bugzilla.mozilla.org/show_bug.cgi?id=1681512))
*   From Firefox 85 it’s also possible to disable tab-to-search globally by unchecking the *Search Engines* checkbox in the *Address Bar Preferences*, under *Privacy & Security*.
*   Firefox now supports printing non-contiguous page ranges (e.g. 1-3, 6, 7) – [Bug 499640](https://bugzilla.mozilla.org/show_bug.cgi?id=499640)
*   DevTools and Marionette are now fully Fission compatible! Congratulations to those teams!
    *   Reminder: Nightly users can help us test Fission by enabling it in about:preferences#experimental, and [filing bugs here](https://bugzilla.mozilla.org/enter_bug.cgi?assigned_to=nobody%40mozilla.org&blocked=1561396&bug_ignored=0&bug_severity=--&bug_status=NEW&bug_type=defect&cf_a11y_review_project_flag=---&cf_fission_milestone=---&cf_fx_iteration=---&cf_fx_points=---&cf_root_cause=---&cf_status_firefox83=---&cf_status_firefox84=---&cf_status_firefox85=---&cf_status_firefox86=---&cf_status_firefox_esr78=---&cf_status_thunderbird_esr78=---&cf_tracking_firefox84=---&cf_tracking_firefox85=---&cf_tracking_firefox86=---&cf_tracking_firefox_esr78=---&cf_tracking_firefox_relnote=---&cf_tracking_firefox_sumo=---&cf_tracking_thunderbird_esr78=---&cf_webcompat_priority=---&component=DOM%3A%20Navigation&contenttypemethod=list&contenttypeselection=text%2Fplain&defined_groups=1&filed_via=standard_form&flag_type-203=X&flag_type-37=X&flag_type-41=X&flag_type-607=X&flag_type-721=X&flag_type-737=X&flag_type-787=X&flag_type-799=X&flag_type-800=X&flag_type-803=X&flag_type-846=X&flag_type-855=X&flag_type-863=X&flag_type-864=X&flag_type-930=X&flag_type-936=X&flag_type-937=X&flag_type-945=X&form_name=enter_bug&maketemplate=Remember%20values%20as%20bookmarkable%20template&op_sys=Unspecified&priority=--&product=Core&rep_platform=Unspecified&target_milestone=---&version=unspecified)

### Friends of the Firefox team

#### Introductions/Shout-Outs

*   \[harry\] Amy Churchwell joins the Search & Navigation team today. She transferred internally from Marketing Engineering. Welcome Amy!

#### [Resolved bugs (excluding employees)](https://bugzilla.mozilla.org/buglist.cgi?title=Resolved%20bugs%20\(excluding%20employees\)&quicksearch=1647931%2C1649618%2C1650645%2C1652412%2C1654217%2C1664768%2C1666831%2C1667061%2C1671579%2C1674806%2C1678173%2C1678372%2C1678616%2C1678865%2C1678866%2C1679252%2C1679412%2C1680931%2C1681213%2C1681554%2C1681642%2C1681948)

#### Fixed more than one bug

*   Masatoshi Kimura \[:emk\]
*   Michelle Goossens \[:masterwayz\]
*   Sonia
*   Tim Nguyen :ntim

#### New contributors (🌟 = first patch)

*   🌟 Ankush Dua [fixed an issue with revoked devtools\_page permissions for WebExtensions](https://bugzilla.mozilla.org/show_bug.cgi?id=1671579)
*   🌟 gero [removed the windowtype attribute](https://bugzilla.mozilla.org/show_bug.cgi?id=1674806) from dialogs where we didn’t need it anymore
*   manekenpix [converted some DevTools code](https://bugzilla.mozilla.org/show_bug.cgi?id=1664768) to use DOM Promises instead of the defer library

### Project Updates

#### Add-ons / Web Extensions

##### Addon Manager & about:addons

*   Starting from Firefox 85, Mozilla-signed privileged addons can be installed from a third party website without triggering the “third party addon install doorhanger” (and without having to add new “install” site permission for those hosts, [e.g. as we had to do for fpn.firefox.com](https://searchfox.org/mozilla-central/rev/23c25cd32a1e87095301273937b4ee162f41e860/browser/app/permissions#24-25)) – [Bug 1681331](https://bugzilla.mozilla.org/show_bug.cgi?id=1681331)

*   Fixed addon startup issue when an extension sideloaded in the profile is updated on disk ([Bug 1664144](https://bugzilla.mozilla.org/show_bug.cgi?id=1664144))
*   Some more small about:addons cleanup from ntim ([Bug 1678173](https://bugzilla.mozilla.org/show_bug.cgi?id=1678173), [Bug 1678865](https://bugzilla.mozilla.org/show_bug.cgi?id=1678865), [Bug 1678866](https://bugzilla.mozilla.org/show_bug.cgi?id=1678866)). Thanks a lot, ntim!

  

##### WebExtensions Framework

*   **Ankush Dua** *contributed a fix for the devtools optional\_permission* (the devtools optional\_permission can be used by extension, like ABP, that provides a devtools panel as a secondary feature of the addon) – [Bug 1671579](https://bugzilla.mozilla.org/show_bug.cgi?id=1671579)
*   Fixed content scripts applied to webpages loaded as subframes of an extension browserAction/pageAction popup when Fission is enabled – [Bug 1680877](https://bugzilla.mozilla.org/show_bug.cgi?id=1680877)
*   Fixed addon startup issue when webRequest is moved from permissions to optional\_permissions in an addon update (regression from [Bug 1624235](https://bugzilla.mozilla.org/show_bug.cgi?id=1624235)) – [Bug 1637059](https://bugzilla.mozilla.org/show_bug.cgi?id=1637059)

#### Developer Tools

*   **DevTools Fission** **M2** – Making DevTools Fission compatible DONE.
    *   [![A table showing the total number of remaining bugs for the MVP to make the DevTools Fission-compatible.](https://3sgkpvh31s44756j71xlti9b-wpengine.netdna-ssl.com/files/2020/12/headlines85_1.png)](https://3sgkpvh31s44756j71xlti9b-wpengine.netdna-ssl.com/files/2020/12/headlines85_1.png)

        Our DevTools are ready for Fission (out-of-process iframes)!
*   **Marionette Fission** – Making Marionette Fission compatible DONE
    *   [![A table showing the total number of remaining bugs for the MVP to make Marionette Fission-compatible.](https://3sgkpvh31s44756j71xlti9b-wpengine.netdna-ssl.com/files/2020/12/headlines85_2.png)](https://3sgkpvh31s44756j71xlti9b-wpengine.netdna-ssl.com/files/2020/12/headlines85_2.png)

        Marionette, the framework that allows Firefox to be tested with automation, is now Fission compatible too!

#### Fission

*   Neil has patches up to [improve the behaviour of the tab unloader](https://bugzilla.mozilla.org/show_bug.cgi?id=1682442), and [show UI when subframes crash](https://bugzilla.mozilla.org/show_bug.cgi?id=1644911)

#### Installer & Updater

*   Background updater work is also proceeding, with [Bug 1676296](https://bugzilla.mozilla.org/show_bug.cgi?id=1676296) landing last week to support managing scheduled tasks in Gecko, and more development still also happening on the background task framework.

#### New Tab Page and Pocket

*   We’re running three experiments:
    *   Newtab Pocket stories in AU and NZ
    *   New signup/login call-to-action in the Pocket doorhanger
    *   We’re testing some changes to newtab story personalization

#### [Password Manager](https://wiki.mozilla.org/Toolkit:Password_Manager)

*   Dimi fixed [Bug 1677710](https://bugzilla.mozilla.org/show_bug.cgi?id=1677710) The password manager code triggers main thread sqlite disk I/O off of the gather-telemetry notification
*   And [Bug 1678200](https://bugzilla.mozilla.org/show_bug.cgi?id=1678200) Remove or update probes expiring in Firefox 86: pwmgr.doorhanger\_submitted#doorhanger\_submitted
*   Thanks for Kenrick95 for fixing [Bug 1678616](https://bugzilla.mozilla.org/show_bug.cgi?id=1678616) about:logins menu problem
*   2021 Planning underway

#### PDFs & Printing

*   mstriemer put a Printing… message in the dialog and hid the popup dialog which showed progress, the cancel button on that dialog caused problems and it looked dated [Bug 1679133](https://bugzilla.mozilla.org/show_bug.cgi?id=1679133)
*   mstriemer hid the print setting that don’t relate to PDFs when a PDF is being printed [Bug 1669725](https://bugzilla.mozilla.org/show_bug.cgi?id=1669725)
*   mstriemer updated the form to be disabled when loading a printer’s settings. Sometimes loading a physical printer’s settings can take a few settings and changes could be lost in this time [Bug 1676388](https://bugzilla.mozilla.org/show_bug.cgi?id=1676388)
*   emalysz made a change to avoid updating the preview for some settings that can’t change the preview output [Bug 1676199](https://bugzilla.mozilla.org/show_bug.cgi?id=1676199)
*   sfoster added a paginator to the preview when it’s hovered to show current page, next/prev/first/last buttons [Bug 1654684](https://bugzilla.mozilla.org/show_bug.cgi?id=1654684)
*   emalysz added support for non-contiguous page ranges (ex: 1-3, 6, 7) [Bug 499640](https://bugzilla.mozilla.org/show_bug.cgi?id=499640)
*   emalysz fixed an issue where the form could get disabled with custom margins interactions [Bug 1674106](https://bugzilla.mozilla.org/show_bug.cgi?id=1674106)

#### Performance

*   emalysz continues to make progress leading the charge migrating us from OS.File to IOUtils
    *   Shout out to barret for landing necessary changes to IOUtils to support the migration!
    *   Currently investigating [a bizarre ts\_paint\_webext regression](https://bugzilla.mozilla.org/show_bug.cgi?id=1679252) caused by one of these conversions
*   emalysz [fixed a performance issue with the Screenshots feature](https://bugzilla.mozilla.org/show_bug.cgi?id=1664444), and made it more compatible with Fission
*   bigiri has [a patch to move SharedDataMap.jsm off of OSFile](https://bugzilla.mozilla.org/show_bug.cgi?id=1649610)
*   florian’s team has [landed some great documentation](https://firefox-source-docs.mozilla.org/tools/profiler/markers-guide.html) for the new profiler marker API
*   florian has some new visualization variations up for the BHR dashboard
    *   [showFrames](http://queze.net/bhr/test/#showFrames=1)
    *   [onlyXulLeaf](http://queze.net/bhr/test/#showFrames=1&onlyXulLeaf=1)
    *   [skipKnownBugs](http://queze.net/bhr/test/#showFrames=1&onlyXulLeaf=1&skipKnownBugs=1)
    *   This BHR dashboard helped identify a hang caused by the password manager code, [which has been recently fixed](https://bugzilla.mozilla.org/show_bug.cgi?id=1677710)! Thanks, dimi!
*   Gijs [made the Bookmarks Toolbar initialization occur later in the startup window](https://bugzilla.mozilla.org/show_bug.cgi?id=1667237)
*   Gijs [fixed some flicker](https://bugzilla.mozilla.org/show_bug.cgi?id=1681169) that occurred when launching the browser with the Bookmarks Toolbar enabled
*   mconley fixed [an AsyncShutdown hang caused by the about:home startup cache](https://bugzilla.mozilla.org/show_bug.cgi?id=1673716)
*   mconley [re-enabled TART](https://bugzilla.mozilla.org/show_bug.cgi?id=1651311)
*   dthayer has [some fixes](https://bugzilla.mozilla.org/show_bug.cgi?id=1672789) and [polish](https://bugzilla.mozilla.org/show_bug.cgi?id=1678488) for [the pre-XUL skeleton UI](https://bugzilla.mozilla.org/show_bug.cgi?id=1680258)

#### Picture-in-Picture

*   We’ve got the green light for another round of MSU students hacking on Picture-in-Picture next semester! mhowell and mtigley will be mentoring them.
*   In progress:
    *   [Bug 1677080 – Fullscreen PiP window is affected by switching video source](https://bugzilla.mozilla.org/show_bug.cgi?id=1677080)
    *   [Bug 1677107 – Add Telemetry for tracking multiple PiP support usage](https://bugzilla.mozilla.org/show_bug.cgi?id=1677107)
    *   [Bug 1681796 – Prevent superfluous PictureInPictureParent actors from being associated with each tab](https://bugzilla.mozilla.org/show_bug.cgi?id=1681796)
    *   [Bug 1680796 – Ensure that the tab’s Toolkit:PictureInPicture actor is destroyed before moving to next test](https://bugzilla.mozilla.org/show_bug.cgi?id=1680796)
    *   [Bug 1678390 – Prevent Picture-in-Picture windows from opening on top of one another](https://bugzilla.mozilla.org/show_bug.cgi?id=1678390)

#### Search and Navigation

*   Fixed regressions related to Input Method Editor, in particular loss of the last token ([Bug 1673669](https://bugzilla.mozilla.org/show_bug.cgi?id=1673669)) and race conditions causing the wrong search engine to be used or Search Mode to be lost ([Bug 1679697](https://bugzilla.mozilla.org/show_bug.cgi?id=1679697), [Bug 1678647](https://bugzilla.mozilla.org/show_bug.cgi?id=1678647))
*   Introduced a new advanced preference to keep the Address Bar results panel open during IME composition. This provides a better experience for keyboard layouts that don’t open a picker panel. In the future we hope to be able to auto-detect that situation, but in the meanwhile, you can flip *browser.urlbar.imeCompositionClosesPanel* to false and test the alternative behavior ([Bug 1673971](https://bugzilla.mozilla.org/show_bug.cgi?id=1673971))
*   URL canonization ([www.\*.com](about:blank)) now uses https by default, the protocol can be customized through the *browser.fixup.alternate.protocol* advanced pref ([Bug 1638215](https://bugzilla.mozilla.org/show_bug.cgi?id=1638215))
*   Work continued on the weather QuickSuggest experiment, but its release has been moved to January.
*   Region.jsm now can use a Geolocation monitor to update without hitting the network ([Bug 1663501](https://bugzilla.mozilla.org/show_bug.cgi?id=1663501))
*   Fixed a bug where search engines were being re-added on startup after their removal, when using a language pack ([Bug 1675624](https://bugzilla.mozilla.org/show_bug.cgi?id=1675624))