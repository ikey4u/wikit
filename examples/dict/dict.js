function opentab(tabButton, tabContentID) {
  // Reset tab button and content to default style
  for (btn of document.getElementsByClassName("tabbtn")) {
        btn.className = "tabbtn";
  }
  for (content of document.getElementsByClassName("tabcontent")) {
        content.className = "tabcontent";
        content.style.display = "none";
  }

  // active current tab button
  tabButton.className = "tabbtn active";

  // active current tab content
  let tabContent = document.getElementById(tabContentID);
  if (tabContent) {
    tabContent.style.display = "block";
    tabContent.className = "tabcontent active";
  }
}
