(function() {var implementors = {};
implementors["lignin"] = [{"text":"impl From&lt;Comment&gt; for Comment","synthetic":false,"types":[]},{"text":"impl&lt;'a&gt; From&lt;&amp;'a Comment&gt; for &amp;'a Comment","synthetic":false,"types":[]},{"text":"impl From&lt;Event&gt; for Event","synthetic":false,"types":[]},{"text":"impl&lt;'a&gt; From&lt;&amp;'a Event&gt; for &amp;'a Event","synthetic":false,"types":[]},{"text":"impl From&lt;HtmlElement&gt; for HtmlElement","synthetic":false,"types":[]},{"text":"impl&lt;'a&gt; From&lt;&amp;'a HtmlElement&gt; for &amp;'a HtmlElement","synthetic":false,"types":[]},{"text":"impl From&lt;SvgElement&gt; for SvgElement","synthetic":false,"types":[]},{"text":"impl&lt;'a&gt; From&lt;&amp;'a SvgElement&gt; for &amp;'a SvgElement","synthetic":false,"types":[]},{"text":"impl From&lt;Text&gt; for Text","synthetic":false,"types":[]},{"text":"impl&lt;'a&gt; From&lt;&amp;'a Text&gt; for &amp;'a Text","synthetic":false,"types":[]},{"text":"impl From&lt;ThreadSafe&gt; for ThreadBound","synthetic":false,"types":[]},{"text":"impl&lt;C&gt; From&lt;CallbackRef&lt;ThreadSafe, C&gt;&gt; for CallbackRef&lt;ThreadBound, C&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;C: CallbackSignature,&nbsp;</span>","synthetic":false,"types":[]},{"text":"impl&lt;R, C&gt; From&lt;&amp;'_ CallbackRegistration&lt;R, C&gt;&gt; for CallbackRef&lt;ThreadSafe, C&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;R: Sync,<br>&nbsp;&nbsp;&nbsp;&nbsp;C: CallbackSignature,&nbsp;</span>","synthetic":false,"types":[]},{"text":"impl&lt;R, C&gt; From&lt;&amp;'_ CallbackRegistration&lt;R, C&gt;&gt; for CallbackRef&lt;ThreadBound, C&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;C: CallbackSignature,&nbsp;</span>","synthetic":false,"types":[]},{"text":"impl&lt;'a&gt; From&lt;Element&lt;'a, ThreadSafe&gt;&gt; for Element&lt;'a, ThreadBound&gt;","synthetic":false,"types":[]},{"text":"impl&lt;'a&gt; From&lt;EventBinding&lt;'a, ThreadSafe&gt;&gt; for EventBinding&lt;'a, ThreadBound&gt;","synthetic":false,"types":[]},{"text":"impl&lt;'a&gt; From&lt;Node&lt;'a, ThreadSafe&gt;&gt; for Node&lt;'a, ThreadBound&gt;","synthetic":false,"types":[]},{"text":"impl&lt;'a&gt; From&lt;ReorderableFragment&lt;'a, ThreadSafe&gt;&gt; for ReorderableFragment&lt;'a, ThreadBound&gt;","synthetic":false,"types":[]},{"text":"impl&lt;'a, S1, S2&gt; From&lt;&amp;'a [Node&lt;'a, S1&gt;]&gt; for Node&lt;'a, S2&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;S1: ThreadSafety + Into&lt;S2&gt;,<br>&nbsp;&nbsp;&nbsp;&nbsp;S2: ThreadSafety,&nbsp;</span>","synthetic":false,"types":[]},{"text":"impl&lt;'a, S1, S2&gt; From&lt;&amp;'a mut [Node&lt;'a, S1&gt;]&gt; for Node&lt;'a, S2&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;S1: ThreadSafety + Into&lt;S2&gt;,<br>&nbsp;&nbsp;&nbsp;&nbsp;S2: ThreadSafety,&nbsp;</span>","synthetic":false,"types":[]},{"text":"impl&lt;'a, S&gt; From&lt;&amp;'a str&gt; for Node&lt;'a, S&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;S: ThreadSafety,&nbsp;</span>","synthetic":false,"types":[]},{"text":"impl&lt;'a, S&gt; From&lt;&amp;'a mut str&gt; for Node&lt;'a, S&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;S: ThreadSafety,&nbsp;</span>","synthetic":false,"types":[]}];
implementors["lignin_html"] = [{"text":"impl&lt;'a, S:&nbsp;ThreadSafety&gt; From&lt;Error&gt; for Error&lt;'a, S&gt;","synthetic":false,"types":[]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()