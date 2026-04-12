[https://purplesyringa.moe/blog/no-one-owes-you-supply-chain-security/](https://purplesyringa.moe/blog/no-one-owes-you-supply-chain-security/)

* * *

## Comments

> · · 76 points
> 
> Open source is not a supply chain.
> 
> A supplier is someone who has a formal relationship with a downstream entity, typically around a certain specification of the product including quality assurance and recourse if that quality is not met. Open source has none of that. The formality of the relationship falls full stop on the license and every single FOSS license has the "this software is provided as is without warranty" clause.
> 
> You get what you get, and every time there is a "supply-chain attack" within open source, the fault lies completely with the downstream software, not the person providing the software.
> 
> I think the concept of free (as in beer) software supply chain security just puts the burden on an already strained group primarily consisting of hobbyists trying to do this *for fun*. And the fingers get pointed the wrong way.
> 
> I believe we can absolutely have supply chain security for free (as in freedom, NOT beer) by *charging for it*. I know there are a handful of for-profit companies that are trying this at scale (e.g. Tidelift). RedHat does this basically for their ecosystem. It seems unpopular because people mix the free as in freedom with free as in beer and get mad when both aren't provided when really the promise of FOSS is completely about freedoms and not at all about beer (i.e. [https://www.fsf.org/blogs/community/free-software-is-not-antithetical-to-commercial-success](https://www.fsf.org/blogs/community/free-software-is-not-antithetical-to-commercial-success)).
> 
> In lieu of that, I think the premise of this article as I read it makes complete sense: an ecosystem can provide the tools for YOU (the downstream) to audit your own dependencies, but the ecosystem itself is not responsible for the \[non-existent\] supply-chain.
> 
> > · · 8 points
> > 
> > > I think the concept of free (as in beer) software supply chain security just puts the burden on an already strained group primarily consisting of hobbyists trying to do this for fun. And the fingers get pointed the wrong way.
> > 
> > I wholeheartedly agree. Framing this as "supply chain security" implicitly assigns responsibility to maintainers who never signed up for that role. That's exactly [what bothered me](https://lucumr.pocoo.org/2022/7/9/congratulations/) about PyPI declaring "critical packages" at one point: it creates a class of projects where additional expectations get imposed purely based on downstream reliance, not any change in the maintainer's intent or capacity.
> > 
> > The reality continues to be that people depend on too many dependencies, they got addicted to package manager just dishing them out for free. When I went into Open Source there were no package manages in the modern sense. There was Debian, and it was a high friction package manager and you talked to the folks packaging up your libraries. If you wanted to depend on something for your project, you were much more likely to vendor it. There was no "supply chain".
> > 
> > I hope the pendulum will start to swing back a bit, at least in so far to make people more aware of the hidden cost of dependencies.
> > 
> > > · · 1 points
> > > 
> > > that the cost of dependencies is too hidden nowadays is fair, but people have been preaching this kind of "dependency veganism" since npm became a thing, and it doesn't seem to me like anything changed since then. which makes me think that "just have fewer dependencies" is an untenable position to have (especially for larger teams), and not actually useful advice. I've worked on codebases with that mantra of vendoring everything and in hindsight it didn't solve anything either.

> · · 28 points
> 
> > Rust gives build scripts and procedural macros full access to your PC. Worse, rust-analyzer runs cargo check when you open the project directory, so it can effectively become a 0-click RCE.
> 
> This is technically documented in the rust-analyzer web site, but of all the rust developers I've mentioned it to, zero of them were aware of the problem! It's especially bad when you consider that the means for determining whether some code is trustworthy is often to *open it in your text editor*, so if that's configured to hit LSP automatically, you're toast.
> 
> The idea that macros should be able to write to arbitrary files on disk is *completely insane* and I really hope future languages learn from these mistakes.
> 
> > · · 9 points
> > 
> > > It's especially bad when you consider that the means for determining whether some code is trustworthy is often to open it in your text editor, so if that's configured to hit LSP automatically, you're toast.
> > 
> > Most editors will ask if you trust the authors of a project before executing arbitrary code from it.
> > 
> > This isn't limited to Rust. If you open a cmake project, the editor will execute arbitrary commands in order to determine what command-line options will be passed to the C compiler. For JVM projects (java, kotlin, scala, even clojure) opening a project in the IDE can trigger arbitrary code from the build system.
> > 
> > > · · 9 points
> > > 
> > > Go is the only language I can name off of the top of my head which guarantees this doesn't happen.
> > > 
> > > The minimal version selection scheme also comes to mind.
> > > 
> > > I'm sure there are other security measures but those are which I'm aware of.
> > > 
> > > · · 3 points
> > > 
> > > > Most editors will ask if you trust the authors of a project before executing arbitrary code from it.
> > > 
> > > Oh, which ones are these? The only one I know which does this is VS Code.
> > > 
> > > > For JVM projects (java, kotlin, scala, even clojure) opening a project in the IDE can trigger arbitrary code from the build system.
> > > 
> > > I've tried to raise this as an issue with the author of clojure-lsp, but he doesn't seem that interested in fixing it: [https://github.com/clojure-lsp/clojure-lsp/issues/1747](https://github.com/clojure-lsp/clojure-lsp/issues/1747) The ironic thing in this case is that the Clojure LSP server bends over backwards to avoid evaluating macro code, but then can be trivially tricked into running arbitrary build code.
> > > 
> > > > · · 4 points
> > > > 
> > > > > The only one I know which does this is VS Code.
> > > > 
> > > > Intellij and Zed do this too, I believe.
> > > > 
> > > > > Clojure LSP server bends over backwards to avoid evaluating macro code
> > > > 
> > > > Neat! But how does it manage to do that without impacting LSP operations? Like, does go-to-definition just fail for identifiers that come from a macro?
> > 
> > > · · 3 points
> > > 
> > > > Most editors will ask if you trust the authors of a project before executing arbitrary code from it.
> > > 
> > > To which the only correct answer would be “no”. Even a project I trust can be compromised by pulling in a new version of a dependency. Nobody is going to manually inspect the source code of every dependency on every git pull. But nobody wants to run their editor with 90% of the features disabled.
> > > 
> > > The only real solution is programming languages (and OSes) that take least privilege seriously: [https://medium.com/agoric/pola-would-have-prevented-the-event-stream-incident-45653ecbda99](https://medium.com/agoric/pola-would-have-prevented-the-event-stream-incident-45653ecbda99)
> 
> > · · 5 points
> > 
> > > This is technically documented in the rust-analyzer web site, but of all the rust developers I've mentioned it to, zero of them were aware of the problem!
> > 
> > I'm confused by that. Providing (paid) support for rust-analyzer, a lot of the people I speak to are
> > 
> > a) well aware of this behaviour b) at the juicy targets, no one cares
> > 
> > b) is because of: you are taking external code into your product and associated infrastructure. It will get executed eventually. build.rs is a vector - it is only *one* vector. The real reason people dislike build.rs is because it breaks a lot of the declarativeness of Rusts build process.
> > 
> > > The idea that macros should be able to write to arbitrary files on disk is completely insane and I really hope future languages learn from these mistakes.
> > 
> > AFAIK, this is not committed nor intended behaviour, but what we are stuck with and rust-analyzer needs to kinda play the game of rustc here, as annoying as it is.
> > 
> > [https://rust-lang.github.io/rfcs/3698-declarative-derive-macros.html](https://rust-lang.github.io/rfcs/3698-declarative-derive-macros.html) may make a class of proc macros declarative and make things much easier for rust-analyzer and other tools.
> > 
> > > · · 1 points
> > > 
> > > A middle point I've been trying to push for is to identify the top 10 uses of build.rs and provide a declarative mechanism for them. A lot of them are checking rustc version or some other minimal platform checks. That one that is hard is any that call to cc because the ones I saw using those couldn't be expressed in a simple declarative way (lots of tweaking based on complex logic).
> > > 
> > > The problem with that idea is that you *still* need the build.rs to exist for backcompat with toolchains that don't yet understand the declarative system.
> > > 
> > > The other approach would be to use the same call-graph analysis mechanism that one would implement in rustc to track transitive calls to panic and generalize it to track other "attributes", like access to filesystem or calling syscalls. If we had that analysis, then in the Cargo.toml a crate could declare that it doesn't access the filesystem. If during compilation access to the filesystem is detected, that'd be an error. This approach has the benefit that it is fully back compatible (other than maybe the Cargo.toml itself having unexpected fields?) and usable for crates, their build.rs and by extension work for proc-macros.
> > > 
> > > This analysis is possible. The problem is writing an appropriate RFC and the implementation being cheap enough to avoid slowdowns on existing crates. The post-mono analysis approach that I borrowed from rust klint when experimenting with redpen works well, but adds time and memory consumption by adding a stage. Another approach would be to include an effect system tracking metadata to body analysis to propagate marks in a table of DefIds to marks as typeck/nameres goes so that we don't another stage. Either way, there are issues when it comes to trait objects and closures, where you have to be able to customize the logic to "assume present" or "assume not present" when encountering objects for traits that are freely implementable outside the current crate or closures of unknown provenance.
> 
> > · · 3 points
> > 
> > I'm well aware of the problem, but such things are super common in build systems. Many build systems invoke `make`, `awk`, `python`, `perl`, `sh`, etc. If you've ever built an entire Linux distro, you're very familiar with the total madness of tools and random internet downloads that happen at build time. Could Rust have done it better? Perhaps. But despite the fact that `build.rs` should be avoided for plain Rust projects, it solves real issues with respect to interoperability and tool calls.
> > 
> > > · · 4 points
> > > 
> > > Which distro are you talking about? Debian doesn't allow any such thing for its builds - the whole distro must be able to be built offline. Are there other distros that don't follow this model?
> > > 
> > > > · · 2 points
> > > > 
> > > > Any distro. They may use predownloaded things or restrict things to known hashes in the build instead of the internet connection, but in practice, there's Perl, awk and sh involved somewhere in your bootstrap process and many many other things. I think building and testing bison (for gcc) required Perl.

> · · 14 points
> 
> Say what one will about “capitalist hellscapes” - there’s something to be said for large programming environments that do pay their developers and are on the hook for the security of their expansive standard library. I’m increasingly preferring .NET and Go precisely because I don’t want to place undue expectations on hardworking volunteers. Until a well funded organisation decides to take on commercial support for a broad swathe of Rust crates these discussions will continue.
> 
> > · · 4 points
> > 
> > Same with Apple and Swift, which may be known just for iOS/Mac apps, but is capable of so much more. And it is a very nice language, too!
> > 
> > > · · 2 points
> > > 
> > > For sure - a few years ago I wrote a Mac Gemini browser for fun (not exactly complicated, granted!) and it was quite refreshing that I didn’t need a single thing outside the provided frameworks. After I crunched down the app icons the installer package was a little under 1MB.

> · · 5 points
> 
> Obviously no one owes you supply chain security, and corporations clearly and continuously abuse (through support demands and blame) unpaid volunteer maintainers.
> 
> The obvious solution for them is to stop using OSS (they won't because so many tech startups and service providers have built an entire business model on selling the work of others, and stealing the work of *every* OSS project to create text predictors), and all the people using text predictors will continue to demand the ability to use OSS code and ignore the licenses of that software, while pretending they're not stealing OSS code.
> 
> But even aside from that: look at how the OSS community attacked the old xz maintainer: how *dare* that volunteer not provide commercial level support to them? sure that maintainer wasn't being paid, but neither were the projects that used xz. So that maintainer *owed* them support and service, and had no right to move on and transfer maintenance to someone who hadn't been subject to an audit and background check.
> 
> So let's look at it simply: If you *choose* to put your project in one of the big package repos you *are* taking on that responsibility. If you don't want that responsibility, don't post the package to npm, cargo, etc.
> 
> Trivially you're right: unpaid volunteers do not *owe* anyone anything. But if they don't want to provide any kind of "I won't turn this into a crypto miner or wallet stealer or whatever" guarantees then they shouldn't be publishing it to services with the specific intent of encouraging people to use it. They can leave their projects off cargo, npm, etc - obviously that would mean they don't get the fame, clout, etc that they're wanting but that's the trade off.
> 
> > · · 4 points
> > 
> > I think you're painting a black-and-white picture that doesn't check out. Just because something is unsupported doesn't mean it doesn't work well enough; you don't have to give your 100% to make the world better. I'd bet something like 90% of the world's software infrastructure relies on unsupported OSS; and if we followed your rules, we'd still be in the computing stone age, strong-armed by corporations.
> > 
> > · · 2 points
> > 
> > > They can leave their projects off cargo, npm, etc - obviously that would mean they don't get the fame, clout, etc that they're wanting but that's the trade off.
> > 
> > It's *slightly* more complicated than that, in that tooling these days is optimized for "import code via online registry". There are means to use your own registry (even as part of version control, e.g. in [forgejo](https://forgejo.org/docs/latest/user/packages/)) so you can use the standard processes with your own private stuff, but I'm not sure how many folks are even aware that, for example, cargo can use [additional registries](https://doc.rust-lang.org/cargo/reference/registries.html).
> > 
> > Go seems to be the one large ecosystem that has this nailed, by using URIs by default.

> · · 5 points
> 
> If crates.io is public infrastructure and it's chronically underfunded, then “audit your own dependencies” is the wrong takeaway. It shifts the cost from the companies that benefit most onto individual teams. A better response is collective funding for crates.io's security work, not making every team repeat the same audit work on its own.
> 
> > · · 11 points
> > 
> > I'm afraid I don't have the political clout to make that happen just by posting :) My intention was to say "no one's paying those people, so don't expect them to do work for you", and let others draw conclusions from that indirectly. But of course you're right.
> > 
> > · · 5 points
> > 
> > An interesting approach comes from [Cargo Vet](https://mozilla.github.io/cargo-vet/). [Their feature to import audits](https://mozilla.github.io/cargo-vet/importing-audits.html) enables any org to publish their list of trusted crates, so I can easily decide to not audit any crate that Mozilla claims to have audited.
> > 
> > An ecosystem of audits can be created on top of such feature.
> > 
> > In general, at work we evaluated Cargo Vet at work and did some work on it- we had a small component were we minimized dependencies and audited those, recording our audit using Cargo Vet. Although we shifted focus from that component, I kinda liked Cargo Vet, but I'm not really knowledgeable about this area.

> · · 5 points
> 
> It cuts both ways.
> 
> On the one hand, when you’re choosing to use some dependency from some unpaid source you have signed no contract with, you are solely responsible for checking that said dependency is fit for purpose, correct enough, and of course not malicious.
> 
> On the other hand, when you chose to *publish* something, it is your moral duty to make sure doing so does not make the world a worse place. That you’re doing more good (utility, safety, entertainment…) than harm (time spent, risks, frustration…). It’s not enough to disclaim all legal responsibility in the licence. In most cases, this means pushing the very *state of the art*, in one aspect or another.
> 
> The infosec community is especially cognisant of that second part. That’s how "don’t roll your own crypto" came to be: in most cases your users would be safer if you sought out a famous library instead. Though some tend to have tunnel vision, and treat cryptographic code like it’s something special. I’m not hearing "don’t write your own server", or "don’t write your own image decoder", or "don’t write your own text editor", yet they too process untrusted inputs.
> 
> I *do* hear "don’t use memory unsafe languages" for such things, but that safety often comes at a cost (run times, memory consumption, time to develop, portability/availability…). I’m pretty sure there are many cases where an unsafe language ultimately provides more net value than a safe one. Though I reckon the gap is gradually closing — which is good.

> · · 3 points
> 
> i think that the huge amount of work which goes into constantly chasing brittle dependency chains in "done" projects is swept under the carpet. supply chain attacks are only one aspect of this problem.
> 
> wouldn't larger, well designed and quasi-complete standard libraries (something like what golang has) and comprehensive libraries in general make the developer's life easier? this is a development culture move, not a technical one. it involves upfront work: be choosy in what you use, occasionally duplicate work, vendor/fork other people's software and so on. what you get is a stable product which doesn't break every time you update the dependencies (because it has very few and most importantly none is a deep chain).
> 
> maybe the bazaar has shown its limits and we need to call the cathedral builders for help in finding a healthy balance.
> 
> > · · 2 points
> > 
> > You don't even need standard libraries or cathedrals. You just need a wide enough acknowledgement that a library that has seen no changes in ten years and still works just as it worked back then is a good thing, actually.

> · · 1 points
> 
> Because of supply chain attacks, I decided to fully isolate and virtualize every development environment that touches anything that doesn't come from my Linux distribution's official repositories. Even made an entire set of virtual machine orchestration scripts to deal with it. If these things somehow find a way to escape the hypervisor, they're probably gonna be big enough problems for the cloud industry to deal with it for us.
> 
> > · · 7 points
> > 
> > This is a tempting way when I think about this problem. Just sandbox development environments.
> > 
> > However, you'd still ship the code to be used outside the sandbox. I think sandboxing prevents malicious dependencies from owning your workstation, but I don't see how it helps you ship stuff that doesn't carry malicious dependencies :(