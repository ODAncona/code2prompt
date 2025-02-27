/*
	Photon by HTML5 UP
	html5up.net | @ajlkn
	Free for personal and commercial use under the CCA 3.0 license (html5up.net/license)
*/

function copyToClipboard() {
	const codeBlock = document.getElementById("code-block").innerText;
	navigator.clipboard.writeText(codeBlock);
}

(function($) {

	var	$window = $(window), $body = $('body');

	// Breakpoints.
	breakpoints({
		xlarge:   [ '1141px',  '1680px' ],
		large:    [ '981px',   '1140px' ],
		medium:   [ '737px',   '980px'  ],
		small:    [ '481px',   '736px'  ],
		xsmall:   [ '321px',   '480px'  ],
		xxsmall:  [ null,      '320px'  ]
	});

	// Play initial animations on page load.
	$window.on('load', function() {
		window.setTimeout(function() {
			$body.removeClass('is-preload');
		}, 100);
	});

	// Scrolly.
	$('.scrolly').scrolly();

	const scrollers = document.querySelectorAll(".scroller");

	// If a user hasn't opted in for recuded motion, then we add the animation
	if (!window.matchMedia("(prefers-reduced-motion: reduce)").matches) {
	  addAnimation();
	}
	
	function addAnimation() {
	  scrollers.forEach((scroller) => {
		// add data-animated="true" to every `.scroller` on the page
		scroller.setAttribute("data-animated", true);
	
		// Make an array from the elements within `.scroller-inner`
		const scrollerInner = scroller.querySelector(".scroller__inner");
		const scrollerContent = Array.from(scrollerInner.children);
	
		// For each item in the array, clone it
		// add aria-hidden to it
		// add it into the `.scroller-inner`
		scrollerContent.forEach((item) => {
		  const duplicatedItem = item.cloneNode(true);
		  duplicatedItem.setAttribute("aria-hidden", true);
		  scrollerInner.appendChild(duplicatedItem);
		});
	  });
	}


})(jQuery);