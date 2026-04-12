[https://purplesyringa.moe/blog/no-one-owes-you-supply-chain-security/](https://purplesyringa.moe/blog/no-one-owes-you-supply-chain-security/)

* * *

## Comments

> · · 17 points
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

> · · 14 points
> 
> > Rust gives build scripts and procedural macros full access to your PC. Worse, rust-analyzer runs cargo check when you open the project directory, so it can effectively become a 0-click RCE.
> 
> This is technically documented in the rust-analyzer web site, but of all the rust developers I've mentioned it to, zero of them were aware of the problem! It's especially bad when you consider that the means for determining whether some code is trustworthy is often to *open it in your text editor*, so if that's configured to hit LSP automatically, you're toast.
> 
> The idea that macros should be able to write to arbitrary files on disk is *completely insane* and I really hope future languages learn from these mistakes.
> 
> > · · 4 points
> > 
> > > It's especially bad when you consider that the means for determining whether some code is trustworthy is often to open it in your text editor, so if that's configured to hit LSP automatically, you're toast.
> > 
> > Most editors will ask if you trust the authors of a project before executing arbitrary code from it.
> > 
> > This isn't limited to Rust. If you open a cmake project, the editor will execute arbitrary commands in order to determine what command-line options will be passed to the C compiler. For JVM projects (java, kotlin, scala, even clojure) opening a project in the IDE can trigger arbitrary code from the build system.
> > 
> > > · · 3 points
> > > 
> > > Go is the only language I can name off of the top of my head which guarantees this doesn't happen.
> > > 
> > > The minimal version selection scheme also comes to mind.
> > > 
> > > I'm sure there are other security measures but those are which I'm aware of.
> > > 
> > > · · 1 points
> > > 
> > > > Most editors will ask if you trust the authors of a project before executing arbitrary code from it.
> > > 
> > > Oh, which ones are these? The only one I know which does this is VS Code.
> > > 
> > > > For JVM projects (java, kotlin, scala, even clojure) opening a project in the IDE can trigger arbitrary code from the build system.
> > > 
> > > I've tried to raise this as an issue with the author of clojure-lsp, but he doesn't seem that interested in fixing it: [https://github.com/clojure-lsp/clojure-lsp/issues/1747](https://github.com/clojure-lsp/clojure-lsp/issues/1747) The ironic thing in this case is that the Clojure LSP server bends over backwards to avoid evaluating macro code, but then can be trivially tricked into running arbitrary build code.
> > > 
> > > > · · 2 points
> > > > 
> > > > > The only one I know which does this is VS Code.
> > > > 
> > > > Intellij and Zed do this too, I believe.
> > > > 
> > > > > Clojure LSP server bends over backwards to avoid evaluating macro code
> > > > 
> > > > Neat! But how does it manage to do that without impacting LSP operations? Like, does go-to-definition just fail for identifiers that come from a macro?
> 
> > · · 2 points
> > 
> > I'm well aware of the problem, but such things are super common in build systems. Many build systems invoke `make`, `awk`, `python`, `perl`, `sh`, etc. If you've ever built an entire Linux distro, you're very familiar with the total madness of tools and random internet downloads that happen at build time. Could Rust have done it better? Perhaps. But despite the fact that `build.rs` should be avoided for plain Rust projects, it solves real issues with respect to interoperability and tool calls.
> > 
> > > · · 2 points
> > > 
> > > Which distro are you talking about? Debian doesn't allow any such thing for its builds - the whole distro must be able to be built offline. Are there other distros that don't follow this model?

> · · 3 points
> 
> Say what one will about “capitalist hellscapes” - there’s something to be said for large programming environments that do pay their developers and are on the hook for the security of their expansive standard library. I’m increasingly preferring .NET and Go precisely because I don’t want to place undue expectations on hardworking volunteers. Until a well funded organisation decides to take on commercial support for a broad swathe of Rust crates these discussions will continue.

> · · 3 points
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