<template>
    <div class="capture-container">
        <div class="header">
            <h1 class="title-capture">Découvrir</h1>
        </div>
        <div class="content">
            <h1 >Gide utilisateur</h1>
            <h1 >Wiki</h1>
            <h1 >Documentation</h1>
            <h1 >Open issue</h1>
            <h1><a href="https://github.com/Sonar-team/Sonar_desktop_app" 
                    target="_blank">
                    GitHub
                </a>
            </h1>        
        </div>
    </div>
</template>

<style scoped>
.capture-container {
    margin: 20px;
    display: inline-block;
flex-direction: column;
background-color: #f6f6f6; /* Adjust the background color as needed */
text-align: left;
margin-right: 20px;
}

.title-capture {
color: #888;
font-family: Oxanium;
font-size: 60px;
font-style: normal;
font-weight: 500;
line-height: normal;  
font-size: 2em; /* Adjust the font size as needed */
color: #333; /* Adjust the text color as needed */
margin: 0 0 10px 0; /* Adjust the margin as needed */
text-align: left;
}

.content {
    display: flex; /* Use flexbox for layout */
    justify-content: space-around; /* Adjust the spacing between items */
    align-items: center; /* Align items vertically */
    color: black;
    font-size: 10px;
    font-family: Oxanium;
    font-weight: 900;
}

</style>
