/* JavaScript file for MicroSoft Bing Dictionary */
(function() {
  var _undefined="undefined",_authid="authid",_crossid="crossid",_homoid="homoid",_webid="webid",_authtabid="authtabid",_crosstabid="crosstabid",_homotabid="homotabid",_webtabid="webtabid",_colid="colid",_synoid="synoid",_antoid="antoid",_coltabid="coltabid",_synotabid="synotabid",_antotabid="antotabid",_newktvid="newktvid",_divtag="DIV",_newLid="newLeId",_confilid="confil",_showfilterid="filshow",_hidefilterid="filhide";
  /* added for GoldenDict id fix */
  var class_authtabid="tb_b_authtabid",class_crosstabid="tb_b_crosstabid",class_homotabid="tb_b_homotabid",class_webtabid="tb_b_webtabid",class_coltabid="tb_b_coltabid",class_synotabid="tb_b_synotabid",class_antotabid="tb_b_antotabid";
  var idfix = document.getElementById(_homotabid)||document.getElementById(_crosstabid);

  setTimeout(function(){
    var tb_div = document.getElementsByClassName("tb_div");
    if(tb_div[0]!=null){
      var tab_syn = tb_div[0].children[0];
    }
    if(tb_div[1]!=null){
      var tab_exp = tb_div[1].children[0];
    }
    if(tab_syn!=null){
      tb_div[0].children[0].style.borderBottom="2px solid rgb(83, 83, 83)";
    }
    if(tab_exp!=null){
      tb_div[1].children[0].style.borderBottom="2px solid rgb(83, 83, 83)";
    }
  } ,100);

  openWd = function(n,t){
    var i=document.getElementById(n),r;
    i!=null&&typeof i!=_undefined&&i.tagName==_divtag&&i.style.display=="none"&&(i.style.display="block",r=document.getElementById(t),r&&(r.className="tg_open"))
  }

  toggleWd = function(n,t){
    var i=document.getElementById(t);
    i!=null&&typeof i!=_undefined&&i.tagName==_divtag&&n!=null&&(i.style.display!="none"?(i.style.display="none",n.className="tg_close"):(i.style.display="block",n.className="tg_open"))
  }

  switchWordsTab = function(n,t,i){
    var r=document.getElementById(_colid),u=document.getElementById(_synoid),f=document.getElementById(_antoid);
    if(idfix!=null){
    var e=document.getElementById(_coltabid),o=document.getElementById(_synotabid),s=document.getElementById(_antotabid);
    }else{
    var e=document.getElementsByClassName(class_coltabid)[0],o=document.getElementsByClassName(class_synotabid)[0],s=document.getElementsByClassName(class_antotabid)[0];
    };



    r!=null&&typeof r!=_undefined&&r.tagName==_divtag&&(n=="col"?(r.style.display="block",r.style.borderBottom="1px solid white",e!=null&&(e.style.borderBottom="2px solid rgb(83, 83, 83)")):(r.style.display="none",r.style.borderBottom="",e!=null&&(e.style.borderBottom="")));
    u!=null&&typeof u!=_undefined&&u.tagName==_divtag&&(n=="syno"?(u.style.display="block",u.style.borderBottom="1px solid white",o!=null&&(o.style.borderBottom="2px solid rgb(83, 83, 83)")):(u.style.display="none",u.style.borderBottom="",o!=null&&(o.style.borderBottom="")));
    f!=null&&typeof f!=_undefined&&f.tagName==_divtag&&(n=="anto"?(f.style.display="block",f.style.borderBottom="1px solid white",s!=null&&(s.style.borderBottom="2px solid rgb(83, 83, 83)")):(f.style.display="none",f.style.borderBottom="",s!=null&&(s.style.borderBottom="")));

    openWd(t,i)
  }



  switchDefiTab = function(n,t,i){
    var r=document.getElementById(_authid),u=document.getElementById(_crossid),f=document.getElementById(_homoid),e=document.getElementById(_webid);
    if(idfix!=null){
      var o=document.getElementById(_authtabid),s=document.getElementById(_crosstabid),h=document.getElementById(_homotabid),c=document.getElementById(_webtabid);
    }else{
      var o=document.getElementsByClassName(class_authtabid)[0],s=document.getElementsByClassName(class_crosstabid)[0],h=document.getElementsByClassName(class_homotabid)[0],c=document.getElementsByClassName(class_webtabid)[0];
    };


    r!=null&&typeof r!=_undefined&&r.tagName==_divtag&&(n=="auth"?(r.style.display="block",r.style.borderBottom="1px solid white",o!=null&&(o.style.borderBottom="2px solid rgb(83, 83, 83)")):(r.style.display="none",r.style.borderBottom="",o!=null&&(o.style.borderBottom="")));
    u!=null&&typeof u!=_undefined&&u.tagName==_divtag&&(n=="cross"?(u.style.display="block",u.style.borderBottom="1px solid white",s!=null&&(s.style.borderBottom="2px solid rgb(83, 83, 83)")):(u.style.display="none",u.style.borderBottom="",s!=null&&(s.style.borderBottom="")));
    f!=null&&typeof f!=_undefined&&f.tagName==_divtag&&(n=="homo"?(f.style.display="block",f.style.borderBottom="1px solid white",h!=null&&(h.style.borderBottom="2px solid rgb(83, 83, 83)")):(f.style.display="none",f.style.borderBottom="",h!=null&&(h.style.borderBottom="")));
    e!=null&&typeof e!=_undefined&&e.tagName==_divtag&&(n=="web"?(e.style.display="block",e.style.borderBottom="1px solid white",c!=null&&(c.style.borderBottom="2px solid rgb(83, 83, 83)")):(e.style.display="none",e.style.borderBottom="",c!=null&&(c.style.borderBottom="")));
    openWd(t,i)
  }

  ExampleSwitch = function(n,t,i,r){
    var f=document.getElementById(n).innerHTML,u;
    f!=null&&typeof f!=_undefined&&(f==i?(document.getElementById(n).innerHTML=r,u=document.getElementsByName(t),u!=null&&typeof u!=_undefined?(u=document.getElementsByTagName(_divtag),ExampleStyleChange2(u,"block",t)):ExampleStyleChange1(u,"block")):(document.getElementById(n).innerHTML=i,u=document.getElementsByName(t),u!=null&&typeof u!=_undefined?(u=document.getElementsByTagName(_divtag),ExampleStyleChange2(u,"none",t)):ExampleStyleChange1(u,"none")))
  }

  ExampleStyleChange1 = function(n,t){
    if(n!=null&&typeof n!=_undefined)for(var i=0;i<n.length;i++)n[i].style.display=t
  }

  ExampleStyleChange2 = function(n,t,i){
    if(n!=null&&typeof n!=_undefined)for(var r=0;r<n.length;r++)n[r].getAttribute("name")==i&&(n[r].style.display=t)
  }

  SenseCollectionSwitch = function(n,t,i){
    var u=document.getElementById(t),r=document.getElementById(i);
    u!=null&&typeof u!=_undefined&&(n.className=="pos_open"?(u.style.display="none",r!=null&&typeof r!=_undefined&&(r.style.display="none"),n.className="pos_close"):(u.style.display="block",r!=null&&typeof r!=_undefined&&(r.style.display="block"),n.className="pos_open"))
  }
})()
