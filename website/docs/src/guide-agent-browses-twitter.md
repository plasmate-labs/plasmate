# Let Your AI Agent Browse X (Twitter) as You

This guide is for anyone who has an AI agent set up through OpenClaw and talks to it on Telegram (or WhatsApp, Discord, etc.). You want your agent to be able to read your X/Twitter timeline, post tweets, and reply to people on your behalf.

No coding required. We will walk through every step.

---

## What You Will Need

- **A computer with Google Chrome** (this is where you will install a small browser add-on)
- **Your OpenClaw agent** already running and responding to you on Telegram
- **An X (Twitter) account** that you can log into

That is it. Your agent's server (the Digital Ocean droplet or wherever it runs) does not need any changes. Everything happens through your Chrome browser and a quick message to your agent.

---

## The Big Picture

Here is what we are doing in plain English:

1. You install a tiny Chrome add-on that can copy your login cookies
2. You log into X.com in Chrome like you normally would
3. You click one button to securely share those login cookies with your agent
4. You tell your agent to install the Plasmate skill (one message)
5. Your agent can now browse X.com as if it were you

Your password is never shared. The cookies are encrypted and stored on your agent's server. When the cookies expire, you just repeat steps 2 and 3.

---

## Step 1: Add the Plasmate Extension to Chrome

Open this link in Chrome:

**[Plasmate Cookie Export on the Chrome Web Store](https://chromewebstore.google.com/detail/plasmate-cookie-export/biapncdddgdcjalalpkbngaabijkhheo)**

Click **"Add to Chrome"** and confirm.

You will see a small Plasmate icon appear in your browser toolbar (it might be hidden behind the puzzle-piece icon; click that to pin it).

> **Is this safe?** Yes. The extension only talks to your own machine (localhost). It has no analytics, no tracking, and no external servers. You can read the privacy policy right on the Chrome Web Store page.

---

## Step 2: Tell Your Agent to Install Plasmate

Open your Telegram chat with your agent and send this message:

> **Install the Plasmate skill from ClawHub**

Your agent knows how to do this. It will install the skill from [clawhub.ai/builder-nc/plasmate](https://clawhub.ai/builder-nc/plasmate), which gives it the ability to fetch web pages using the Plasmate browser engine.

Wait for your agent to confirm the install is complete before moving on.

> **What just happened?** Your agent now has a new tool called Plasmate that can read web pages much more efficiently than a regular browser. But it is not logged into anything yet.

---

## Step 3: Start the Cookie Bridge

You need to start a small local server on your computer so the Chrome extension can securely pass cookies to your agent. Your agent can help with this too.

Send your agent this message:

> **I need to set up Plasmate auth. Can you start the auth bridge?**

If your agent is running on a remote server, it will run `plasmate auth serve` there. If it needs you to do something on your end, it will tell you.

You should see a confirmation that the bridge is running on port 9271.

> **Alternatively**, if you are comfortable with a terminal, open one and type:
> ```
> plasmate auth serve
> ```

---

## Step 4: Log Into X.com

1. Open a new tab in Chrome
2. Go to [x.com](https://x.com)
3. Log in with your username and password, just like you normally would
4. Make sure you can see your timeline (scroll around a bit to confirm the session is active)

---

## Step 5: Push Your Login to Plasmate

This is the magic step:

1. **Click the Plasmate icon** in your Chrome toolbar
2. The extension automatically detects that you are on X.com
3. It shows you the cookies it will share (things like `auth_token` and `ct0`)
4. **Click "Push to Plasmate"**
5. You will see a green checkmark. Done!

Your login session has been securely encrypted and stored. Your agent can now use it.

---

## Step 6: Try It Out

Go back to Telegram and send your agent a message like:

> **Browse my X timeline and tell me what people are talking about today**

Or:

> **Check my X mentions and summarize any replies to my recent posts**

Or even:

> **Post a tweet that says: "Beautiful morning! Working on some exciting AI projects today."**

Your agent will use Plasmate with your stored login to access X.com, read the content, and respond to you with what it found.

---

## That's It!

Your agent can now browse X.com as you. Here is what to know going forward:

### It Works for Other Sites Too

The same process works for any website. Want your agent to browse GitHub? LinkedIn? Reddit? Just:

1. Log into that site in Chrome
2. Click the Plasmate extension
3. Click "Push to Plasmate"
4. Tell your agent to use it

### When to Re-authenticate

Login cookies expire over time (usually every few weeks for X.com). If your agent tells you it cannot access a site anymore, just repeat Steps 4 and 5: log in again in Chrome and push the cookies.

### Your Privacy

- Your password is **never** shared with the agent
- Only session cookies are stored, and they are **encrypted**
- Everything stays on your machine and your agent's server
- No data goes to any third party
- You can delete a stored login anytime by telling your agent: *"Remove my X.com auth profile"*

---

## Troubleshooting

**"My agent says it does not know about Plasmate"**
Make sure Step 2 completed successfully. Ask your agent: *"Do you have the Plasmate skill installed?"*

**"The extension says the bridge is not running"**
The Plasmate auth bridge needs to be running for the extension to push cookies. Ask your agent to start it, or run `plasmate auth serve` in a terminal.

**"My agent can see X.com but the content looks like a logged-out page"**
Your cookies may have expired. Repeat Steps 4 and 5 to push fresh ones.

**"I do not see the Plasmate icon in Chrome"**
Click the puzzle-piece icon in Chrome's toolbar and pin the Plasmate extension.

---

## Want to Learn More?

- [Authenticated Browsing (Technical Guide)](guide-authenticated-browsing) for developers who want the full details
- [Plasmate Documentation](overview) to understand how the browser engine works
- [ClawHub Plasmate Skill](https://clawhub.ai/builder-nc/plasmate) to see what your agent can do with Plasmate
